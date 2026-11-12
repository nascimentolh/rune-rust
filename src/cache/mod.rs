//! Reading the game cache from disk.
//!
//! Layers, bottom to top:
//! - `store`           — the files on disk (data file + index files, 520-byte sectors)
//! - `container`       — compression around each archive (none / bzip2 / gzip)
//! - `reference_table` — per-index list of archives, CRCs, versions and files
//! - `archive`         — splitting an archive into its files
//! - `sprite`, `font_metrics`, `text`, `loading_screens`, `interface`, `cs2` — file formats inside archives
//! - `Cache` (here)    — the files on disk plus every index's reference table
//!
//! Checked against the reference client's source and the build-718 cache (see MECHANICS.md).

pub mod archive;
mod buffer;
pub mod container;
pub mod cs2;
mod cs2_opcodes;
pub mod entity_defaults;
pub mod enums;
mod error;
pub mod font_metrics;
pub mod interface;
pub mod keys;
pub mod loading_screens;
pub mod mesh;
pub mod midi;
pub mod music;
pub mod music_patch;
pub mod params;
pub mod reference_table;
pub mod sound_effect;
pub mod sprite;
pub mod store;
pub mod text;
pub mod varbit;
pub mod vorbis;
pub mod writer;
pub mod xtea;

pub use error::{CacheError, Result};
pub use reference_table::ReferenceTable;
pub use store::{FileStore, META_INDEX};

use std::path::Path;

pub struct Cache {
    pub store: FileStore,
    /// Reference table per index, filled in by `load_table`.
    tables: Vec<Option<ReferenceTable>>,
}

impl Cache {
    pub fn open(dir: &Path) -> Result<Self> {
        let store = FileStore::open(dir)?;
        let tables = vec![None; store.index_count()];
        Ok(Cache { store, tables })
    }

    pub fn index_count(&self) -> usize {
        self.store.index_count()
    }

    /// Reads and decodes the reference table of one index (from index 255).
    pub fn load_table(&mut self, index: u8) -> Result<&ReferenceTable> {
        let raw = self.store.read(META_INDEX, index as u32)?;
        let table = ReferenceTable::decode(&container::decompress(&raw)?)?;
        self.tables[index as usize] = Some(table);
        Ok(self.tables[index as usize].as_ref().unwrap())
    }

    pub fn table(&self, index: u8) -> Option<&ReferenceTable> {
        self.tables.get(index as usize)?.as_ref()
    }

    /// Reads one archive and splits it into its files (the table must be loaded).
    pub fn read_files(&mut self, index: u8, archive: u32) -> Result<Vec<Vec<u8>>> {
        let count = self
            .table(index)
            .and_then(|t| t.archives.iter().find(|a| a.id == archive))
            .map(|a| a.files.len())
            .ok_or_else(|| CacheError::Missing(format!("archive {index}/{archive}")))?;
        let raw = self.store.read(index, archive)?;
        archive::split(&container::decompress(&raw)?, count)
    }
}
