//! Start-up, step by step so the screen can show progress:
//! open the cache files, read each index's reference table, then check
//! every map key by decrypting the archive it belongs to.

use crate::cache::container::decrypt_and_decompress;
use crate::cache::keys::MapKeys;
use crate::cache::Cache;
use std::path::PathBuf;

/// Map keys checked per frame.
const KEYS_PER_STEP: usize = 250;

pub enum Stage {
    Opening,
    Tables { next: u8 },
    Keys { next: usize },
    Done,
    Failed(String),
}

/// What was found for one index.
pub struct IndexInfo {
    pub index: u8,
    /// Archive count, file count and revision — or why the table couldn't be read.
    pub result: Result<(usize, usize, i32), String>,
}

/// Result of checking the map keys.
#[derive(Default)]
pub struct KeyReport {
    pub total: usize,
    /// Archive decrypted and decompressed fine.
    pub ok: usize,
    /// Archive is in the cache but didn't decrypt with this key.
    pub wrong: usize,
    /// Archive isn't in this cache.
    pub missing: usize,
    /// Why the keys file couldn't be read, if it couldn't.
    pub error: Option<String>,
}

pub struct Loader {
    pub dir: PathBuf,
    keys_file: PathBuf,
    pub cache: Option<Cache>,
    pub keys: MapKeys,
    pub stage: Stage,
    pub indexes: Vec<IndexInfo>,
    pub key_report: KeyReport,
}

impl Loader {
    pub fn new(dir: PathBuf, keys_file: PathBuf) -> Self {
        Loader {
            dir,
            keys_file,
            cache: None,
            keys: MapKeys::default(),
            stage: Stage::Opening,
            indexes: Vec::new(),
            key_report: KeyReport::default(),
        }
    }

    pub fn finished(&self) -> bool {
        matches!(self.stage, Stage::Done | Stage::Failed(_))
    }

    /// Does one unit of work (one index, or one batch of keys, per call).
    pub fn step(&mut self) {
        self.stage = match std::mem::replace(&mut self.stage, Stage::Done) {
            Stage::Opening => match Cache::open(&self.dir) {
                Ok(cache) => {
                    self.cache = Some(cache);
                    Stage::Tables { next: 0 }
                }
                Err(e) => Stage::Failed(e.to_string()),
            },
            Stage::Tables { next } => self.load_table(next),
            Stage::Keys { next } => self.check_keys(next),
            done => done,
        };
    }

    fn load_table(&mut self, next: u8) -> Stage {
        let cache = self.cache.as_mut().unwrap();
        if next as usize >= cache.index_count() {
            return self.open_keys();
        }
        if cache.store.has_index(next) {
            let result = cache.load_table(next).map(|t| (t.archives.len(), t.file_count(), t.revision)).map_err(|e| e.to_string());
            self.indexes.push(IndexInfo { index: next, result });
        }
        Stage::Tables { next: next + 1 }
    }

    fn open_keys(&mut self) -> Stage {
        match MapKeys::load(&self.keys_file) {
            Ok(keys) => {
                self.key_report.total = keys.len();
                self.keys = keys;
                Stage::Keys { next: 0 }
            }
            Err(e) => {
                self.key_report.error = Some(e.to_string());
                Stage::Done
            }
        }
    }

    fn check_keys(&mut self, next: usize) -> Stage {
        let store = &mut self.cache.as_mut().unwrap().store;
        let end = (next + KEYS_PER_STEP).min(self.keys.entries.len());
        for entry in &self.keys.entries[next..end] {
            match store.read(entry.archive, entry.group) {
                Err(_) => self.key_report.missing += 1,
                Ok(raw) if decrypt_and_decompress(&raw, &entry.key).is_ok() => self.key_report.ok += 1,
                Ok(_) => self.key_report.wrong += 1,
            }
        }
        if end >= self.keys.entries.len() {
            Stage::Done
        } else {
            Stage::Keys { next: end }
        }
    }

    /// Revision of each index's reference table (`None` where it couldn't be read).
    pub fn revisions(&self) -> Vec<Option<i32>> {
        let cache = self.cache.as_ref();
        (0..cache.map_or(0, |c| c.index_count())).map(|i| cache.and_then(|c| c.table(i as u8)).map(|t| t.revision)).collect()
    }

    /// 0..1 for the progress bar: tables are the first half, keys the second.
    pub fn progress(&self) -> f32 {
        match (&self.stage, &self.cache) {
            (Stage::Tables { next }, Some(c)) if c.index_count() > 0 => 0.5 * *next as f32 / c.index_count() as f32,
            (Stage::Keys { next }, _) if self.key_report.total > 0 => 0.5 + 0.5 * *next as f32 / self.key_report.total as f32,
            (Stage::Done, _) => 1.0,
            _ => 0.0,
        }
    }

    pub fn status(&self) -> String {
        match &self.stage {
            Stage::Opening => "Opening cache files...".into(),
            Stage::Tables { next } => format!("Loading reference table {next}..."),
            Stage::Keys { next } => format!("Checking map keys {next}/{}...", self.key_report.total),
            Stage::Done => {
                let ok = self.indexes.iter().filter(|i| i.result.is_ok()).count();
                format!("Loaded {ok} of {} indexes.", self.indexes.len())
            }
            Stage::Failed(e) => format!("Failed: {e}"),
        }
    }
}
