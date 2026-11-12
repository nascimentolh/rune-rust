//! The files on disk: one data file split into 520-byte sectors, and one
//! index file per index whose 6-byte entries say where each archive starts.

use super::error::{corrupt, CacheError, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub const DATA_FILE: &str = "main_file_cache.dat2";
pub const META_INDEX: u8 = 255;
pub const SECTOR_SIZE: usize = 520;
pub const INDEX_ENTRY_SIZE: usize = 6;
/// Archives above this id use the long (10-byte) sector header.
pub const SMALL_ARCHIVE_MAX: u32 = 65535;
const SMALL_HEADER: usize = 8;
const LARGE_HEADER: usize = 10;

pub fn index_file_name(index: u8) -> String {
    format!("main_file_cache.idx{index}")
}

pub struct FileStore {
    data: File,
    /// idx0..idx254, `None` where the file doesn't exist.
    indexes: Vec<Option<File>>,
    meta: File,
}

impl FileStore {
    pub fn open(dir: &Path) -> Result<Self> {
        let open = |name: &str| File::open(dir.join(name));
        let data = open(DATA_FILE).map_err(|_| CacheError::Missing(format!("{DATA_FILE} in {}", dir.display())))?;
        let meta = open(&index_file_name(META_INDEX))
            .map_err(|_| CacheError::Missing(format!("{} in {}", index_file_name(META_INDEX), dir.display())))?;
        let mut indexes: Vec<Option<File>> = (0..META_INDEX).map(|i| open(&index_file_name(i)).ok()).collect();
        while matches!(indexes.last(), Some(None)) {
            indexes.pop();
        }
        Ok(FileStore { data, indexes, meta })
    }

    /// Number of index files (idx0..idxN-1), counting gaps.
    pub fn index_count(&self) -> usize {
        self.indexes.len()
    }

    pub fn has_index(&self, index: u8) -> bool {
        index == META_INDEX || self.indexes.get(index as usize).is_some_and(|f| f.is_some())
    }

    fn index_file(&self, index: u8) -> Result<&File> {
        self.indexes.get(index as usize).and_then(|f| f.as_ref()).ok_or_else(|| CacheError::Missing(format!("index {index}")))
    }

    /// Reads one archive's raw bytes (still compressed), following its sector chain.
    pub fn read(&mut self, index: u8, archive: u32) -> Result<Vec<u8>> {
        let missing = || CacheError::Missing(format!("archive {index}/{archive}"));
        let mut entry = [0u8; INDEX_ENTRY_SIZE];
        let mut idx = if index == META_INDEX { &self.meta } else { self.index_file(index)? };
        idx.seek(SeekFrom::Start(archive as u64 * INDEX_ENTRY_SIZE as u64))?;
        idx.read_exact(&mut entry).map_err(|_| missing())?;
        let size = u24(&entry[0..3]) as usize;
        let mut sector = u24(&entry[3..6]);
        if size == 0 || sector == 0 {
            return Err(missing());
        }

        let header_len = if archive > SMALL_ARCHIVE_MAX { LARGE_HEADER } else { SMALL_HEADER };
        let mut out = Vec::with_capacity(size);
        let mut buf = [0u8; SECTOR_SIZE];
        let mut chunk = 0u16;
        while out.len() < size {
            let take = (size - out.len()).min(SECTOR_SIZE - header_len);
            self.data.seek(SeekFrom::Start(sector as u64 * SECTOR_SIZE as u64))?;
            self.data.read_exact(&mut buf[..header_len + take])?;

            let (h_archive, rest) = if header_len == LARGE_HEADER {
                (u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]), &buf[4..])
            } else {
                (u16::from_be_bytes([buf[0], buf[1]]) as u32, &buf[2..])
            };
            let h_chunk = u16::from_be_bytes([rest[0], rest[1]]);
            let next = u24(&rest[2..5]);
            let h_index = rest[5];
            if h_archive != archive || h_chunk != chunk || h_index != index {
                return corrupt(format!(
                    "sector {sector} of {index}/{archive} belongs to {h_index}/{h_archive} chunk {h_chunk} (expected chunk {chunk})"
                ));
            }
            out.extend_from_slice(&buf[header_len..header_len + take]);
            sector = next;
            chunk = chunk.wrapping_add(1);
        }
        Ok(out)
    }
}

fn u24(b: &[u8]) -> u32 {
    (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32
}
