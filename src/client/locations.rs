//! The locations (objects) of the scene around the player: each region's `l{x}_{y}` decrypted
//! with the key `REGION` gave for it (`MapRegion`), placed as `Class329_Sub1.method12461` does:
//! region tile + (region × 64 − scene base), kept only strictly inside the scene (1..103 both
//! ways); each with its object definition (index 16).
//!
//! `scenery` builds the models of those on every plane placed as `Class329_Sub1.method12459` does,
//! at the middle of the tiles they cover, at the average height of the four corners there:
//! walls (0..=3, 9; a whole corner is two pieces, `method12464`), wall decorations (4..=8, drawn
//! with the type-4 models and stood off the tile's wall by its displacement, `method12465`),
//! scenery (10, 11: type 10 turned another eighth), roofs (12..=21) and ground decoration (22,
//! unless the preference hides plain ones).
//!
//! Not yet: hiding the planes above the player, clipping, animations and transforms.

use super::ground::{regions, Ground, Light, SCENE};
use super::interface::assets::Assets;
use super::object_model::{self, Model, Textures};
use super::scene::{wall_orientation, Piece, Slots, Spot};
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

/// `Class329_Sub1.anIntArray7724` / `anIntArray7720`: how far a straight wall decoration (and a
/// still diagonal outside one) stands off the wall, by rotation, × the displacement.
const STRAIGHT_OFF: [[i32; 2]; 4] = [[1, 0], [0, -1], [-1, 0], [0, 1]];
/// `anIntArray7721` / `anIntArray7713`: the same for the diagonal ones.
const DIAGONAL_OFF: [[i32; 2]; 4] = [[1, -1], [-1, -1], [-1, 1], [1, 1]];

/// The models one location adds: (model type, rotation, offset x, offset y), per
/// `method12459` / `method12464` / `method12465`; `wall` is the wall already on its tile.
fn pieces(kind: u8, rotation: i32, def: &ObjectDef, wall: Option<&ObjectDef>, detail: &Detail) -> Vec<(i8, i32, i32, i32)> {
    let still = !def.dynamic;
    let off = |table: &[[i32; 2]; 4], d: i32| (d * table[rotation as usize][0], d * table[rotation as usize][1]);
    match kind {
        // Walls: a whole corner is the mirrored piece and the next quarter's.
        0 | 1 | 3 | 9 => vec![(kind as i8, rotation, 0, 0)],
        2 => vec![(2, rotation + 4, 0, 0), (2, (rotation + 1) & 3, 0, 0)],
        // Wall decorations are all drawn with the type-4 models (`method8010`).
        4 => vec![(4, rotation, 0, 0)],
        5 => {
            let d = wall.map_or(65, |w| w.decor_displacement + 1);
            let (ox, oy) = off(&STRAIGHT_OFF, d);
            vec![(4, rotation, ox, oy)]
        }
        6 => {
            let d = wall.map_or(33, |w| w.decor_displacement / 2 + 1);
            // The still and the animated decoration use different tables in the reference.
            let (ox, oy) = off(if still { &STRAIGHT_OFF } else { &DIAGONAL_OFF }, d);
            vec![(4, rotation + 4, ox, oy)]
        }
        7 => vec![(4, ((rotation + 2) & 3) + 4, 0, 0)],
        8 => {
            let d = wall.map_or(33, |w| w.decor_displacement / 2 + 1);
            let (ox, oy) = off(&DIAGONAL_OFF, d);
            vec![(4, rotation + 4, ox, oy), (4, ((rotation + 2) & 3) + 4, 0, 0)]
        }
        10 | 12..=21 => vec![(kind as i8, rotation, 0, 0)],
        11 => vec![(10, rotation + 4, 0, 0)],
        22 if detail.ground_decor || def.interactable != 0 || def.clip_type == 1 || def.obstructs_ground => vec![(22, rotation, 0, 0)],
        _ => Vec::new(),
    }
}

/// `method12459` for the locations of `placed` on every plane, in file order: one lit piece per
/// model with the tiles it stands on, and the walls and scenery each tile slot holds (for the
/// roof flood).
pub fn scenery(
    placed: &[Placed],
    assets: &mut Assets,
    ground: &Ground,
    light: &Light,
    brightness: f64,
    detail: &Detail,
) -> (Vec<Piece>, Slots) {
    let mut meshes: HashMap<i32, Option<Mesh>> = HashMap::new();
    let mut models: HashMap<(u32, i8, i32), Option<Model>> = HashMap::new();
    let looks = Textures { table: assets.texture_details(), off: !detail.textures };
    // `SceneObjectManager.getWall`: the last wall placed on each tile so far.
    let mut walls: HashMap<(u8, usize, usize), Rc<ObjectDef>> = HashMap::new();
    let mut out = Vec::new();
    let mut slots = Slots::default();
    let n = SCENE as i32;
    for p in placed {
        let (def, kind, rotation, plane) = (&p.def, p.location.kind, p.location.rotation as i32, p.location.plane);
        if !(detail.textures || !def.hidden) {
            continue;
        }
        let (sx, sy) = if rotation == 1 || rotation == 3 { (def.size.1, def.size.0) } else { def.size };
        let (x, y) = (p.x as i32, p.y as i32);
        let (x0, x1) = if sx + x <= n { (x + (sx >> 1), x + ((sx + 1) >> 1)) } else { (x, x + 1) };
        let (y0, y1) = if y + sy <= n { (y + (sy >> 1), y + ((sy + 1) >> 1)) } else { (y, y + 1) };
        let h = |cx: i32, cy: i32| ground.corner_height(plane as usize, cx as usize, cy as usize);
        let height = (h(x0, y0) + h(x1, y0) + h(x0, y1) + h(x1, y1)) >> 2;
        let at = [(x << 9) + (sx << 8), height, (y << 9) + (sy << 8)];
        let slot = ground.slot(plane as usize, p.x, p.y);
        // Scenery and roofs (and type 9) cover tiles; the rest stand on their own.
        let span = match kind {
            9 => Some([p.x, p.x, p.y, p.y]),
            10..=21 => Some([p.x, (x + sx - 1).clamp(0, n - 1) as usize, p.y, (y + sy - 1).clamp(0, n - 1) as usize]),
            _ => None,
        };
        let spot = match span {
            Some([x0, x1, y0, y1]) => Spot::Span { slot, x0, x1, y0, y1 },
            None => Spot::Tile { slot, x: (at[0] >> 9).clamp(0, n - 1) as usize, y: (at[2] >> 9).clamp(0, n - 1) as usize },
        };
        let placing = pieces(kind, rotation, def, walls.get(&(plane, p.x, p.y)).map(|w| w.as_ref()), detail);
        if placing.is_empty() {
            continue;
        }
        for (model_kind, turn, ox, oy) in placing {
            let built = models.entry((p.location.id, model_kind, turn)).or_insert_with(|| {
                Model::build(def, model_kind, turn, &looks, |id| meshes.entry(id).or_insert_with(|| assets.mesh(id as u32)).clone())
            });
            let Some(mut model) = built.clone() else { continue };
            model.fit(def, ground, plane as usize, at);
            // A decoration's offset moves it when drawn, after the fitting (`GraphNode_Sub1_Sub4`).
            let (vertices, textures) = object_model::vertices(&model.corners([at[0] + ox, at[1], at[2] + oy], light, brightness, &looks));
            out.push(Piece { spot, vertices, textures });
        }
        // The tile slots the flood reads (`method3395` walls, `method3397` scenery).
        match (kind, span) {
            (0 | 1 | 3, _) => slots.at(slot, p.x, p.y).walls = [wall_orientation(kind, rotation), 0],
            (2, _) => slots.at(slot, p.x, p.y).walls = [wall_orientation(2, rotation + 4), wall_orientation(2, (rotation + 1) & 3)],
            (_, Some(s)) => {
                let code = (if kind == 21 { 19 } else { kind }) | ((rotation as u8) << 6);
                for tx in s[0]..=s[1] {
                    for ty in s[2]..=s[3] {
                        slots.at(ground.slot(plane as usize, tx, ty), tx, ty).scenery.push((code, s));
                    }
                }
            }
            _ => {}
        }
        if kind <= 3 {
            walls.insert((plane, p.x, p.y), Rc::clone(def));
        }
    }
    (out, slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walls_and_decorations_pick_their_pieces() {
        let def = ObjectDef::default();
        let detail = Detail { ground_decor: true, textures: true };
        // A whole corner at rotation 3: the mirrored piece (3 + 4) and the next quarter (0).
        assert_eq!(pieces(2, 3, &def, None, &detail), vec![(2, 7, 0, 0), (2, 0, 0, 0)]);
        // A straight outside decoration stands off a wall of displacement 64 by 65, facing
        // rotation 1: (0, −65).
        let wall = ObjectDef::default();
        assert_eq!(pieces(5, 1, &def, Some(&wall), &detail), vec![(4, 1, 0, -65)]);
        // With no wall it stands off by 65 too; a diagonal one by 33.
        assert_eq!(pieces(6, 0, &def, None, &detail), vec![(4, 4, 33, 0)]);
        assert_eq!(pieces(11, 2, &def, None, &detail), vec![(10, 6, 0, 0)]);
    }
}
