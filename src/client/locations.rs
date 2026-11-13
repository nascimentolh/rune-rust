//! The locations (objects) of the scene around the player: each region's `l{x}_{y}` decrypted
//! with the key `REGION` gave for it (`MapRegion`), placed as `Class329_Sub1.method12461` does:
//! region tile + (region × 64 − scene base), kept only strictly inside the scene (1..103 both
//! ways); each with its object definition (index 16).
//!
//! `scenery` builds the models of those on plane 0 placed as `Class329_Sub1.method12459` does:
//! scenery (types 10, 11), roofs (12..=21) and ground decoration (22, unless the preference
//! hides plain ones) at the middle of the tiles they cover, at the average height of the four
//! corners there; type 11 is type 10 turned another eighth (`GraphNode_Sub1_Sub1_Sub6`).
//!
//! Not yet: walls and wall decorations (types 0..=9, `method12464` / `method12465`), the planes
//! above, clipping, animations and transforms.

use super::ground::{regions, Ground, Light, SCENE};
use super::interface::assets::Assets;
use super::object_model::Model;
use crate::cache::map_locations::Location;
use crate::cache::mesh::Mesh;
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

/// The game's preferences `scenery` follows.
pub struct Detail {
    /// `groundDecor` != 0: plain ground decoration is shown.
    pub ground_decor: bool,
    /// `textures` != 0: `hidden` objects are shown.
    pub textures: bool,
}

/// `method12459` for the plane-0 scenery, roofs and ground decoration of `placed`, lit and
/// coloured: triangle corners (x, height, y) with RGBA.
pub fn scenery(
    placed: &[Placed],
    assets: &mut Assets,
    ground: &Ground,
    light: &Light,
    brightness: f64,
    detail: &Detail,
) -> Vec<([i32; 3], [u8; 4])> {
    let mut meshes: HashMap<i32, Option<Mesh>> = HashMap::new();
    let mut models: HashMap<(u32, i8, i32), Option<Model>> = HashMap::new();
    let mut out = Vec::new();
    let n = SCENE as i32;
    for p in placed.iter().filter(|p| p.location.plane == 0) {
        let (def, kind, rotation) = (&p.def, p.location.kind as i8, p.location.rotation as i32);
        if !(detail.textures || !def.hidden) {
            continue;
        }
        // The model's type and rotation (`GraphNode_Sub1_Sub1_Sub6.method16117`).
        let (kind, turn) = match kind {
            22 if detail.ground_decor || def.interactable != 0 || def.clip_type == 1 || def.obstructs_ground => (22, rotation),
            10 | 12..=21 => (kind, rotation),
            11 => (10, rotation + 4),
            _ => continue,
        };
        let (sx, sy) = if rotation == 1 || rotation == 3 { (def.size.1, def.size.0) } else { def.size };
        let (x, y) = (p.x as i32, p.y as i32);
        let (x0, x1) = if sx + x <= n { (x + (sx >> 1), x + ((sx + 1) >> 1)) } else { (x, x + 1) };
        let (y0, y1) = if y + sy <= n { (y + (sy >> 1), y + ((sy + 1) >> 1)) } else { (y, y + 1) };
        let h = |cx: i32, cy: i32| ground.corner_height(cx as usize, cy as usize);
        let height = (h(x0, y0) + h(x1, y0) + h(x0, y1) + h(x1, y1)) >> 2;
        let at = [(x << 9) + (sx << 8), height, (y << 9) + (sy << 8)];
        let built = models
            .entry((p.location.id, kind, turn))
            .or_insert_with(|| Model::build(def, kind, turn, |id| meshes.entry(id).or_insert_with(|| assets.mesh(id as u32)).clone()));
        let Some(mut model) = built.clone() else { continue };
        model.fit(def, ground, at);
        out.extend(model.corners(at, light, brightness));
    }
    out
}
