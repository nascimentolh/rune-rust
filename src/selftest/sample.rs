//! A small sample cache covering every format case, used when the real cache
//! isn't available: protocols 5/6/7, no/gzip/bzip2 compression, multi-file
//! archives, a large archive id, and an XTEA-encrypted map archive with its key.

use crate::cache::container::{compress, Compression};
use crate::cache::reference_table::{ArchiveEntry, FileEntry, ReferenceTable, FLAG_NAMED, FLAG_WHIRLPOOL};
use crate::cache::writer::CacheWriter;
use crate::cache::{archive, xtea, META_INDEX};
use std::path::{Path, PathBuf};

pub const MAP_INDEX: u8 = 5;
const MAP_KEY: xtea::Key = [0x1234, -0x5678, 0x9abc, -0xdef0];

/// How one index of the sample is built.
struct IndexLayout {
    index: u8,
    protocol: u8,
    flags: u8,
    kind: Compression,
    archive_ids: &'static [u32],
    files_per_archive: usize,
    /// Bytes per file.
    size: usize,
}

/// Writes the sample cache into `dir/cache` and its keys into `dir/keys.json`.
/// Returns (cache folder, keys file).
pub fn write(dir: &Path) -> std::io::Result<(PathBuf, PathBuf)> {
    let mut w = CacheWriter::new();
    let layouts = [
        IndexLayout { index: 0, protocol: 5, flags: 0, kind: Compression::None, archive_ids: &[0, 1, 5], files_per_archive: 1, size: 40 },
        IndexLayout {
            index: 2,
            protocol: 6,
            flags: FLAG_NAMED,
            kind: Compression::Gzip,
            archive_ids: &[0, 10],
            files_per_archive: 3,
            size: 300,
        },
        IndexLayout {
            index: MAP_INDEX,
            protocol: 6,
            flags: FLAG_NAMED,
            kind: Compression::Gzip,
            archive_ids: &[1],
            files_per_archive: 1,
            size: 900,
        },
        IndexLayout {
            index: 7,
            protocol: 7,
            flags: FLAG_NAMED | FLAG_WHIRLPOOL,
            kind: Compression::Bzip2,
            archive_ids: &[3, 70_000],
            files_per_archive: 1,
            size: 5000,
        },
    ];
    for IndexLayout { index, protocol, flags, kind, archive_ids, files_per_archive, size } in layouts {
        let named = flags & FLAG_NAMED != 0;
        let mut entries = Vec::new();
        for &a in archive_ids {
            let files: Vec<Vec<u8>> =
                (0..files_per_archive).map(|f| (0..size).map(|i| ((i * 31 + a as usize + f * 7) % 256) as u8).collect()).collect();
            let mut raw = compress(&archive::join(&files), kind);
            if index == MAP_INDEX {
                xtea::encrypt(&mut raw[5..], &MAP_KEY);
            }
            w.put(index, a, &raw);
            entries.push(ArchiveEntry {
                id: a,
                name_hash: named.then_some(a as i32),
                whirlpool: (flags & FLAG_WHIRLPOOL != 0).then_some([0; 64]),
                crc: crc32fast::hash(&raw) as i32,
                version: 1,
                files: (0..files_per_archive as u32).map(|f| FileEntry { id: f * 2, name_hash: named.then_some(f as i32) }).collect(),
            });
        }
        let table = ReferenceTable { protocol, revision: 718, flags, archives: entries };
        w.put(META_INDEX, index as u32, &compress(&table.encode(), Compression::Gzip));
    }
    let cache_dir = dir.join("cache");
    w.save(&cache_dir)?;

    let keys_file = dir.join("keys.json");
    let [a, b, c, d] = MAP_KEY;
    std::fs::write(
        &keys_file,
        format!(r#"[{{"archive": {MAP_INDEX}, "group": 1, "name": "l50_50", "mapsquare": 12850, "key": [{a}, {b}, {c}, {d}]}}]"#),
    )?;
    Ok((cache_dir, keys_file))
}
