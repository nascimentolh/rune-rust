//! Reference tables: stored in index 255, one per index. They list every
//! archive in that index with its CRC, version and the files inside it.
//!
//! Layout (decompressed):
//!   protocol u8 (5, 6 or 7) · [revision i32 if protocol >= 6] · flags u8
//!   archive count (u16, or big smart for protocol 7) · archive ids (delta-coded)
//!   [name hash i32 per archive, if flags & NAMED]
//!   [64-byte whirlpool per archive, if flags & WHIRLPOOL]
//!   crc i32 per archive · version i32 per archive
//!   file count per archive · file ids per archive (delta-coded)
//!   [file name hash i32 per file, if flags & NAMED]

use super::buffer::Reader;
use super::error::{corrupt, Result};

pub const FLAG_NAMED: u8 = 0x1;
pub const FLAG_WHIRLPOOL: u8 = 0x2;

#[derive(Clone, Debug, PartialEq)]
pub struct FileEntry {
    pub id: u32,
    pub name_hash: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveEntry {
    pub id: u32,
    pub name_hash: Option<i32>,
    pub whirlpool: Option<[u8; 64]>,
    pub crc: i32,
    pub version: i32,
    pub files: Vec<FileEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceTable {
    pub protocol: u8,
    pub revision: i32,
    pub flags: u8,
    /// Sorted by archive id.
    pub archives: Vec<ArchiveEntry>,
}

impl ReferenceTable {
    pub fn file_count(&self) -> usize {
        self.archives.iter().map(|a| a.files.len()).sum()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let protocol = r.u8()?;
        if !(5..=7).contains(&protocol) {
            return corrupt(format!("unknown reference table protocol {protocol}"));
        }
        let revision = if protocol >= 6 { r.i32()? } else { 0 };
        let flags = r.u8()?;
        if flags & !(FLAG_NAMED | FLAG_WHIRLPOOL) != 0 {
            return corrupt(format!("unsupported reference table flags {flags:#x}"));
        }
        let named = flags & FLAG_NAMED != 0;
        let count = |r: &mut Reader| -> Result<u32> {
            if protocol >= 7 {
                r.big_smart()
            } else {
                Ok(r.u16()? as u32)
            }
        };

        let n = count(&mut r)? as usize;
        let ids = read_delta_ids(&mut r, n, &count)?;
        let names = if named { Some(read_i32s(&mut r, n)?) } else { None };
        let whirlpools = if flags & FLAG_WHIRLPOOL != 0 {
            Some((0..n).map(|_| Ok(r.bytes(64)?.try_into().unwrap())).collect::<Result<Vec<[u8; 64]>>>()?)
        } else {
            None
        };
        let crcs = read_i32s(&mut r, n)?;
        let versions = read_i32s(&mut r, n)?;
        let file_counts = (0..n).map(|_| count(&mut r)).collect::<Result<Vec<u32>>>()?;
        let file_ids = file_counts.iter().map(|&c| read_delta_ids(&mut r, c as usize, &count)).collect::<Result<Vec<_>>>()?;
        let file_names =
            if named { Some(file_counts.iter().map(|&c| read_i32s(&mut r, c as usize)).collect::<Result<Vec<_>>>()?) } else { None };

        let archives = (0..n)
            .map(|a| ArchiveEntry {
                id: ids[a],
                name_hash: names.as_ref().map(|v| v[a]),
                whirlpool: whirlpools.as_ref().map(|v| v[a]),
                crc: crcs[a],
                version: versions[a],
                files: file_ids[a]
                    .iter()
                    .enumerate()
                    .map(|(f, &id)| FileEntry { id, name_hash: file_names.as_ref().map(|v| v[a][f]) })
                    .collect(),
            })
            .collect();
        Ok(ReferenceTable { protocol, revision, flags, archives })
    }

    /// The inverse of `decode` (used to build test caches).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![self.protocol];
        let count = |out: &mut Vec<u8>, v: u32| {
            if self.protocol >= 7 && v > 0x7fff {
                out.extend((v | 0x8000_0000).to_be_bytes());
            } else {
                out.extend((v as u16).to_be_bytes());
            }
        };
        if self.protocol >= 6 {
            out.extend(self.revision.to_be_bytes());
        }
        out.push(self.flags);
        count(&mut out, self.archives.len() as u32);
        write_delta_ids(&mut out, self.archives.iter().map(|a| a.id), &count);
        for a in &self.archives {
            if let Some(h) = a.name_hash {
                out.extend(h.to_be_bytes());
            }
        }
        for a in &self.archives {
            if let Some(w) = a.whirlpool {
                out.extend(w);
            }
        }
        self.archives.iter().for_each(|a| out.extend(a.crc.to_be_bytes()));
        self.archives.iter().for_each(|a| out.extend(a.version.to_be_bytes()));
        self.archives.iter().for_each(|a| count(&mut out, a.files.len() as u32));
        for a in &self.archives {
            write_delta_ids(&mut out, a.files.iter().map(|f| f.id), &count);
        }
        for f in self.archives.iter().flat_map(|a| &a.files) {
            if let Some(h) = f.name_hash {
                out.extend(h.to_be_bytes());
            }
        }
        out
    }
}

fn read_i32s(r: &mut Reader, n: usize) -> Result<Vec<i32>> {
    (0..n).map(|_| r.i32()).collect()
}

/// Ids are stored as differences from the previous id.
fn read_delta_ids(r: &mut Reader, n: usize, count: &dyn Fn(&mut Reader) -> Result<u32>) -> Result<Vec<u32>> {
    let mut last = 0u32;
    (0..n)
        .map(|_| {
            last = last.wrapping_add(count(r)?);
            Ok(last)
        })
        .collect()
}

fn write_delta_ids(out: &mut Vec<u8>, ids: impl Iterator<Item = u32>, count: &dyn Fn(&mut Vec<u8>, u32)) {
    let mut last = 0;
    for id in ids {
        count(out, id - last);
        last = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_for_every_protocol() {
        for (protocol, flags) in [(5, 0), (6, FLAG_NAMED), (7, FLAG_NAMED | FLAG_WHIRLPOOL)] {
            let named = flags & FLAG_NAMED != 0;
            let table = ReferenceTable {
                protocol,
                revision: if protocol >= 6 { 42 } else { 0 },
                flags,
                archives: [0u32, 3, 40000]
                    .iter()
                    .map(|&id| ArchiveEntry {
                        id,
                        name_hash: named.then_some(id as i32 * 31),
                        whirlpool: (flags & FLAG_WHIRLPOOL != 0).then_some([id as u8; 64]),
                        crc: -5,
                        version: 9,
                        files: vec![FileEntry { id: 0, name_hash: named.then_some(1) }, FileEntry { id: 2, name_hash: named.then_some(2) }],
                    })
                    .collect(),
            };
            assert_eq!(ReferenceTable::decode(&table.encode()).unwrap(), table);
        }
    }
}
