//! Clip flags: what blocks walking on each tile (`ClipFlagMap`, Darkan's `CollisionMap`), and how
//! a map region's terrain and locations set them (`Class329.decodeTileMasksUnderlay` /
//! `initClipMap`, `Class329_Sub1.method12459` / `method12464`).
//!
//! The client keeps one `ClipFlagMap` per plane for its scene. Here the flags are kept a map
//! region (64 × 64 tiles, 4 planes) at a time, in tiles of the whole map, so a server can hold
//! the regions its players walk in. A region is built on its own (`region_clip`: a list of
//! flags to add, which may reach a tile or two into the neighbours) and put in with `insert`;
//! flags for a neighbour not held yet wait for it.
//!
//! Left out from the client's version, which are drawing choices: objects hidden with textures
//! off (`defs.hidden`), ground decorations left out by the preference, and the roof hiding
//! option (`aBool7717`).

use crate::cache::map_locations::Location;
use crate::cache::map_terrain::Terrain;
use crate::cache::object_def::ObjectDef;
use std::collections::HashMap;

/// `ClipFlag`: the walls' directions (`NW_OBJ` 0x1 … `W_OBJ` 0x80), shifted by 0 for walls, 9
/// for walls that block projectiles (`*_BOUND`) and 22 for the walls the route finder tests
/// (`*_ALT_OBJ`).
pub const NW: i32 = 0x1;
pub const N: i32 = 0x2;
pub const NE: i32 = 0x4;
pub const E: i32 = 0x8;
pub const SE: i32 = 0x10;
pub const S: i32 = 0x20;
pub const SW: i32 = 0x40;
pub const W: i32 = 0x80;
const BOUND: u32 = 9;
pub const ALT: u32 = 22;
/// `OBJ`, `BLOCKED_OBJ` (it blocks projectiles too), `ALT_OBJ`: a solid object on the tile.
const OBJ: i32 = 0x100;
const BLOCKED_OBJ: i32 = 0x20000;
pub const ALT_OBJ: i32 = 0x40000000;
/// `BLOCKED_DECO`: a ground decoration that blocks walking.
pub const BLOCKED_DECO: i32 = 0x40000;
/// `BLOCKED`: the tile itself can't be walked on.
pub const BLOCKED: i32 = 0x200000;
/// `ClipFlag.EMPTY`: every flag, what the client's map edges hold (and a region not held).
pub const EVERYTHING: i32 = -1;

/// A tile's region id (`x >> 6 << 8 | y >> 6`).
pub fn region_of(x: i32, y: i32) -> u32 {
    ((x >> 6) << 8 | (y >> 6)) as u32
}

/// A region's flags to add, in tiles of the whole map, and whether it has terrain (without it,
/// every tile stays blocked: the client clears `BLOCKED` only where terrain was read).
pub struct RegionClip {
    pub id: u32,
    pub has_terrain: bool,
    pub flags: Vec<(usize, i32, i32, i32)>,
}

/// The clip flags of the regions held.
#[derive(Default)]
pub struct Collision {
    regions: HashMap<u32, Vec<i32>>,
    /// Flags for regions not held yet, from their neighbours' objects.
    waiting: HashMap<u32, Vec<(usize, i32, i32, i32)>>,
}

fn slot(plane: usize, x: i32, y: i32) -> usize {
    plane * 64 * 64 + (x & 63) as usize * 64 + (y & 63) as usize
}

impl Collision {
    /// The flags of tile (x, y) on `plane`; every flag when its region isn't held.
    pub fn flags(&self, plane: usize, x: i32, y: i32) -> i32 {
        match self.regions.get(&region_of(x, y)) {
            Some(tiles) if plane < 4 => tiles[slot(plane, x, y)],
            _ => EVERYTHING,
        }
    }

    pub fn holds(&self, region: u32) -> bool {
        self.regions.contains_key(&region)
    }

    /// Puts a region in: its tiles start clear (`unblock`) when it has terrain, else blocked,
    /// then its flags and those its neighbours left for it are added; its flags for a neighbour
    /// go there, or wait for it.
    pub fn insert(&mut self, clip: RegionClip) {
        let start = if clip.has_terrain { 0 } else { BLOCKED };
        self.regions.insert(clip.id, vec![start; 4 * 64 * 64]);
        for (plane, x, y, flag) in self.waiting.remove(&clip.id).into_iter().flatten().chain(clip.flags) {
            match self.regions.get_mut(&region_of(x, y)) {
                Some(tiles) => tiles[slot(plane, x, y)] |= flag,
                None => self.waiting.entry(region_of(x, y)).or_default().push((plane, x, y, flag)),
            }
        }
    }
}

/// `flagBoundary` (`addWall`): a wall of `kind` 0 (straight), 1 / 3 (diagonal, corner) or 2
/// (whole corner) at `rotation`: its own tile's direction(s) and the side it faces on the
/// neighbour, as walls, as projectile blockers when it `blocks`, and as the route finder's.
fn wall(out: &mut Vec<(usize, i32, i32, i32)>, plane: usize, (x, y): (i32, i32), kind: u8, rotation: u8, blocks: bool) {
    let sides: &[(i32, i32, i32)] = match (kind, rotation & 3) {
        (0, 0) => &[(0, 0, W), (-1, 0, E)],
        (0, 1) => &[(0, 0, N), (0, 1, S)],
        (0, 2) => &[(0, 0, E), (1, 0, W)],
        (0, _) => &[(0, 0, S), (0, -1, N)],
        (1 | 3, 0) => &[(0, 0, NW), (-1, 1, SE)],
        (1 | 3, 1) => &[(0, 0, NE), (1, 1, SW)],
        (1 | 3, 2) => &[(0, 0, SE), (1, -1, NW)],
        (1 | 3, _) => &[(0, 0, SW), (-1, -1, NE)],
        (_, 0) => &[(0, 0, N | W), (-1, 0, E), (0, 1, S)],
        (_, 1) => &[(0, 0, N | E), (0, 1, S), (1, 0, W)],
        (_, 2) => &[(0, 0, E | S), (1, 0, W), (0, -1, N)],
        (_, _) => &[(0, 0, S | W), (0, -1, N), (-1, 0, E)],
    };
    let shifts: &[u32] = if blocks { &[0, BOUND, ALT] } else { &[0, ALT] };
    for &shift in shifts {
        for &(dx, dy, flag) in sides {
            out.push((plane, x + dx, y + dy, flag << shift));
        }
    }
}

/// `flagBlockedObject` (`addObject`): every tile a `size` object covers from (x, y).
fn object(out: &mut Vec<(usize, i32, i32, i32)>, plane: usize, (x, y): (i32, i32), size: (i32, i32), blocks: bool) {
    let flag = OBJ | if blocks { BLOCKED_OBJ } else { 0 } | ALT_OBJ;
    for tx in x..x + size.0 {
        for ty in y..y + size.1 {
            out.push((plane, tx, ty, flag));
        }
    }
}

/// Region (rx, ry)'s flags: its blocked tiles (`initClipMap`: settings bit 0x1, on the plane
/// below when the tile is under a bridge, `tileMasks[1] & 0x2`), then each location's
/// (`method12459`, on the plane below under a bridge): a ground decoration with clip type 1
/// blocks its tile; walls (0..=3) flag their sides; a wall object (9), scenery (10, 11) and
/// roofs (12..=21) flag the tiles they cover (their size turned with them); wall decorations
/// (4..=8) and objects with clip type 0 (`ignoreAltClip` makes it 0) nothing. `def` gives an
/// object's definition.
pub fn region_clip(
    rx: i32,
    ry: i32,
    terrain: Option<&Terrain>,
    locations: &[Location],
    mut def: impl FnMut(u32) -> Option<ObjectDef>,
) -> RegionClip {
    let (bx, by) = (rx << 6, ry << 6);
    let mut flags = Vec::new();
    let bridge = |x: usize, y: usize| terrain.is_some_and(|t| t.tile(1, x, y).settings & 0x2 != 0);
    if let Some(t) = terrain {
        for plane in 0..4 {
            for x in 0..64 {
                for y in 0..64 {
                    if t.tile(plane, x, y).settings & 0x1 != 0 {
                        let below = if bridge(x, y) { plane.checked_sub(1) } else { Some(plane) };
                        if let Some(p) = below {
                            flags.push((p, bx + x as i32, by + y as i32, BLOCKED));
                        }
                    }
                }
            }
        }
    }
    for loc in locations {
        let plane = usize::from(loc.plane);
        let Some(plane) = (if bridge(usize::from(loc.x), usize::from(loc.y)) { plane.checked_sub(1) } else { Some(plane) }) else {
            continue;
        };
        let Some(d) = def(loc.id) else { continue };
        let at = (bx + i32::from(loc.x), by + i32::from(loc.y));
        let size = if loc.rotation & 1 == 1 { (d.size.1, d.size.0) } else { d.size };
        match loc.kind {
            22 if d.clip_type == 1 => flags.push((plane, at.0, at.1, BLOCKED_DECO)),
            0..=3 if d.clip_type != 0 => wall(&mut flags, plane, at, loc.kind, loc.rotation, d.blocks),
            9..=21 if d.clip_type != 0 => object(&mut flags, plane, at, size, d.blocks),
            _ => {}
        }
    }
    RegionClip { id: (rx << 8 | ry) as u32, has_terrain: terrain.is_some(), flags }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wall_flags_both_sides_and_waits_for_a_missing_neighbour() {
        let mut flags = Vec::new();
        // A straight wall on the west side of (64, 10), region 1/0: the east side of (63, 10),
        // in region 0/0, gets flagged too.
        wall(&mut flags, 0, (64, 10), 0, 0, true);
        let mut c = Collision::default();
        c.insert(RegionClip { id: region_of(64, 10), has_terrain: true, flags });
        assert_eq!(c.flags(0, 64, 10), W | W << BOUND | W << ALT);
        assert_eq!(c.flags(0, 63, 10), EVERYTHING);
        c.insert(RegionClip { id: region_of(63, 10), has_terrain: true, flags: Vec::new() });
        assert_eq!(c.flags(0, 63, 10), E | E << BOUND | E << ALT);
        assert_eq!(c.flags(0, 62, 10), 0);
    }

    #[test]
    fn objects_turn_with_their_rotation_and_no_terrain_blocks() {
        let mut flags = Vec::new();
        object(&mut flags, 1, (5, 5), (2, 1), false);
        let mut c = Collision::default();
        c.insert(RegionClip { id: 0, has_terrain: true, flags });
        assert_eq!(c.flags(1, 6, 5), OBJ | ALT_OBJ);
        assert_eq!(c.flags(1, 5, 6), 0);
        c.insert(RegionClip { id: region_of(200, 0), has_terrain: false, flags: Vec::new() });
        assert_eq!(c.flags(0, 200, 0), BLOCKED);
    }
}
