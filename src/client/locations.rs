//! The locations (objects) of the scene around the player: each region's `l{x}_{y}` decrypted
//! with the key `REGION` gave for it (`MapRegion`), placed as `Class329_Sub1.method12461` does:
//! region tile + (region × 64 − scene base), kept only strictly inside the scene (1..103 both
//! ways); each with its object definition (index 16).
//!
//! Not yet: building and drawing their models (`ObjectDefinition.method7971`,
//! `Class329_Sub1.method12459`), clipping, animations and transforms.

use super::ground::{regions, SCENE};
use super::interface::assets::Assets;
use crate::cache::map_locations::Location;
use crate::cache::object_def::ObjectDef;
use crate::cache::xtea::Key;
use std::collections::HashMap;
use std::rc::Rc;

/// A location placed in the scene.
pub struct Placed {
    /// Its tile in the scene (0..104).
    pub x: usize,
    pub y: usize,
    pub location: Location,
    pub def: Rc<ObjectDef>,
}

/// Loads the locations of the scene around chunk `chunk`; `keys` in `regions`' order.
pub fn load(assets: &mut Assets, chunk: (i32, i32), keys: &[Key]) -> Vec<Placed> {
    let (list, base) = regions(chunk.0, chunk.1);
    let mut defs: HashMap<u32, Rc<ObjectDef>> = HashMap::new();
    let mut out = Vec::new();
    for (i, (rx, ry)) in list.into_iter().enumerate() {
        let key = keys.get(i).copied().unwrap_or_default();
        let Some(found) = assets.locations(rx, ry, &key) else { continue };
        let (ox, oy) = (rx * 64 - base.0, ry * 64 - base.1);
        for location in found {
            let (sx, sy) = (ox + location.x as i32, oy + location.y as i32);
            let n = SCENE as i32;
            if !(sx > 0 && sy > 0 && sx < n - 1 && sy < n - 1) {
                continue;
            }
            let def = defs.entry(location.id).or_insert_with(|| Rc::new(assets.object(location.id).unwrap_or_default())).clone();
            out.push(Placed { x: sx as usize, y: sy as usize, location, def });
        }
    }
    out
}
