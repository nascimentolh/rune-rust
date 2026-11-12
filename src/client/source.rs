//! Where the client gets cache archives after start-up: the live JS5 session
//! (`updater::Session`) or, for checks without a server, the local cache. `Archives`
//! keeps what was fetched, like the reference's per-index file caches.

use crate::cache::Cache;
use std::collections::HashMap;
use std::rc::Rc;

/// An archive's files by file id; ids the reference table skips are `None`.
pub type Files = Vec<Option<Vec<u8>>>;

/// Places files (in table order) at their ids.
pub fn by_id(ids: &[u32], files: Vec<Vec<u8>>) -> Files {
    let mut out = vec![None; ids.iter().max().map_or(0, |&m| m as usize + 1)];
    for (&id, data) in ids.iter().zip(files) {
        out[id as usize] = Some(data);
    }
    out
}

pub trait Source {
    /// The files of one archive, or `None` if it isn't in the cache.
    fn files(&mut self, index: u8, archive: u32) -> Option<Files>;
}

impl Source for Cache {
    fn files(&mut self, index: u8, archive: u32) -> Option<Files> {
        if self.table(index).is_none() {
            self.load_table(index).ok()?;
        }
        let ids: Vec<u32> = self.table(index)?.archives.iter().find(|a| a.id == archive)?.files.iter().map(|f| f.id).collect();
        Some(by_id(&ids, self.read_files(index, archive).ok()?))
    }
}

pub struct Archives {
    source: Box<dyn Source>,
    fetched: HashMap<(u8, u32), Option<Rc<Files>>>,
}

impl Archives {
    pub fn new(source: Box<dyn Source>) -> Self {
        Archives { source, fetched: HashMap::new() }
    }

    /// The files of an archive, fetched once.
    pub fn archive(&mut self, index: u8, archive: u32) -> Option<Rc<Files>> {
        let source = &mut self.source;
        self.fetched.entry((index, archive)).or_insert_with(|| source.files(index, archive).map(Rc::new)).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_land_at_their_ids() {
        assert_eq!(by_id(&[0, 2], vec![vec![1], vec![2]]), vec![Some(vec![1]), None, Some(vec![2])]);
    }
}
