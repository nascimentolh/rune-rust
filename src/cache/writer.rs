//! Writes a cache in the same on-disk layout `FileStore` reads.
//! Only used to build sample caches for the self-test and unit tests.

use super::store::*;
use std::fs;
use std::io::Result;
use std::path::Path;

#[derive(Default)]
pub struct CacheWriter {
    data: Vec<u8>,
    /// Index id → its index file contents.
    indexes: std::collections::BTreeMap<u8, Vec<u8>>,
}

impl CacheWriter {
    pub fn new() -> Self {
        // Sector 0 is never used, so the data file starts with one empty sector.
        CacheWriter { data: vec![0; SECTOR_SIZE], ..Default::default() }
    }

    /// Stores raw (already containerised) archive bytes in consecutive sectors.
    pub fn put(&mut self, index: u8, archive: u32, raw: &[u8]) {
        let large = archive > SMALL_ARCHIVE_MAX;
        let header_len = if large { 10 } else { 8 };
        let first_sector = (self.data.len() / SECTOR_SIZE) as u32;
        let chunks: Vec<&[u8]> = raw.chunks(SECTOR_SIZE - header_len).collect();
        for (n, chunk) in chunks.iter().enumerate() {
            let sector = first_sector + n as u32;
            let next = if n + 1 < chunks.len() { sector + 1 } else { 0 };
            if large {
                self.data.extend(archive.to_be_bytes());
            } else {
                self.data.extend((archive as u16).to_be_bytes());
            }
            self.data.extend((n as u16).to_be_bytes());
            self.data.extend(&next.to_be_bytes()[1..]);
            self.data.push(index);
            self.data.extend_from_slice(chunk);
            self.data.resize(self.data.len().next_multiple_of(SECTOR_SIZE), 0);
        }

        let idx = self.indexes.entry(index).or_default();
        let at = archive as usize * INDEX_ENTRY_SIZE;
        if idx.len() < at + INDEX_ENTRY_SIZE {
            idx.resize(at + INDEX_ENTRY_SIZE, 0);
        }
        idx[at..at + 3].copy_from_slice(&(raw.len() as u32).to_be_bytes()[1..]);
        idx[at + 3..at + 6].copy_from_slice(&first_sector.to_be_bytes()[1..]);
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(dir.join(DATA_FILE), &self.data)?;
        for (index, bytes) in &self.indexes {
            fs::write(dir.join(index_file_name(*index)), bytes)?;
        }
        Ok(())
    }
}
