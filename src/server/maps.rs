//! The map service: reads map regions from the cache on its own thread and works out their clip
//! flags (`crate::map::collision`), so the world's thread, which may not wait on files, asks for a
//! region and picks it up on a later tick. One per world (two worlds share nothing); each opens
//! the cache for itself.
//!
//! A region is its terrain (`m{x}_{y}` in index 5) and its locations (`l{x}_{y}`, decrypted with
//! the region's map key), with the object definitions they name (index 16). A region the cache
//! doesn't have comes back without terrain: every tile blocked, as the client leaves it.

use crate::cache::map_locations;
use crate::cache::map_terrain::{archive_name, name_hash, Terrain};
use crate::cache::object_def::ObjectDef;
use crate::cache::xtea::Key;
use crate::cache::Cache;
use crate::map::collision::{region_clip, RegionClip};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

const MAPS: u8 = 5;
const OBJECTS: u8 = 16;

/// The world's handle on its map thread.
pub struct MapService {
    requests: Sender<u32>,
    loaded: Receiver<RegionClip>,
}

impl MapService {
    /// Starts the thread on the cache in `dir`, with the map keys by region. It ends when the
    /// handle is dropped.
    pub fn start(dir: PathBuf, keys: Arc<HashMap<u32, Key>>) -> Self {
        let (requests, asked) = channel::<u32>();
        let (done, loaded) = channel();
        std::thread::spawn(move || {
            let Ok(mut cache) = Cache::open(&dir) else { return };
            let mut objects = HashMap::new();
            for region in asked {
                let clip = read_region(&mut cache, &keys, region, &mut objects);
                if done.send(clip).is_err() {
                    return;
                }
            }
        });
        MapService { requests, loaded }
    }

    /// Asks for region `id` (x << 8 | y).
    pub fn request(&self, id: u32) {
        let _ = self.requests.send(id);
    }

    /// A region read since the last call, if any; never waits.
    pub fn take(&self) -> Option<RegionClip> {
        self.loaded.try_recv().ok()
    }
}

/// Region `id` read on the calling thread, from the cache in `dir` (for the self-test, which
/// checks routes against the same flags the world uses).
pub fn read(dir: &std::path::Path, keys: &HashMap<u32, Key>, id: u32) -> Option<RegionClip> {
    let mut cache = Cache::open(dir).ok()?;
    Some(read_region(&mut cache, keys, id, &mut HashMap::new()))
}

/// The archive of index 5 with this name, with its table loaded.
fn named(cache: &mut Cache, name: &str) -> Option<u32> {
    if cache.table(MAPS).is_none() {
        cache.load_table(MAPS).ok()?;
    }
    let hash = name_hash(name);
    cache.table(MAPS)?.archives.iter().find(|a| a.name_hash == Some(hash)).map(|a| a.id)
}

/// Object `id`'s definition (archive id >> 8, the file id & 0xff by its id in the table).
fn object(cache: &mut Cache, id: u32) -> Option<ObjectDef> {
    if cache.table(OBJECTS).is_none() {
        cache.load_table(OBJECTS).ok()?;
    }
    let archive = id >> 8;
    let position = cache.table(OBJECTS)?.archives.iter().find(|a| a.id == archive)?.files.iter().position(|f| f.id == id & 0xff)?;
    let files = cache.read_files(OBJECTS, archive).ok()?;
    ObjectDef::decode(files.get(position)?).ok()
}

fn read_region(cache: &mut Cache, keys: &HashMap<u32, Key>, id: u32, objects: &mut HashMap<u32, Option<ObjectDef>>) -> RegionClip {
    let (rx, ry) = ((id >> 8) as i32, (id & 0xff) as i32);
    let terrain = named(cache, &archive_name(true, rx, ry))
        .and_then(|a| cache.read_files(MAPS, a).ok()?.into_iter().next())
        .and_then(|data| Terrain::decode(&data, rx, ry).ok());
    let key = keys.get(&id).copied().unwrap_or_default();
    let locations = named(cache, &archive_name(false, rx, ry))
        .and_then(|a| cache.read_files_keyed(MAPS, a, &key).ok()?.into_iter().next())
        .and_then(|data| map_locations::decode(&data).ok())
        .unwrap_or_default();
    region_clip(rx, ry, terrain.as_ref(), &locations, |object_id| {
        objects.entry(object_id).or_insert_with(|| object(cache, object_id)).clone()
    })
}
