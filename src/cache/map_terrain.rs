//! A map region's terrain: file 0 of archive `m{x}_{y}` in index 5 (`MapRegion.getArchiveName`).
//! Port of `Class329.decodeTileMasks` (one tile: 4 planes × 64 × 64, plane first, then x, then
//! y) and `CutsceneObject.calculateTileHeights` (the height of a tile the map doesn't give).
//!
//! Left for later: what follows the tiles in the file (atmosphere and lights,
//! `Class329_Sub1.method12471`).

use super::buffer::Reader;
use super::error::Result;

/// Tiles across a region.
pub const SIZE: usize = 64;
/// Planes (heights of the world).
pub const PLANES: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    /// `tileHeights`: negative is up (`-height × 8 << 2`).
    pub height: i32,
    /// `overlayIds` (0 none), `overlayPathShapes`, `overlayRotations`.
    pub overlay: u8,
    pub shape: u8,
    pub rotation: u8,
    /// `regionMap.tileMasks`: the tile's settings (bit 1 blocked, bit 2 bridge...).
    pub settings: u8,
    /// `underlayIds` (0 none).
    pub underlay: u8,
}

/// `[plane][x][y]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Terrain {
    pub tiles: Vec<Tile>,
}

impl Terrain {
    pub fn tile(&self, plane: usize, x: usize, y: usize) -> &Tile {
        &self.tiles[(plane * SIZE + x) * SIZE + y]
    }

    /// The terrain of region (`rx`, `ry`): absolute tile (`rx` × 64 + x, `ry` × 64 + y). A file
    /// cut short leaves the remaining tiles empty, at their noise heights.
    pub fn decode(data: &[u8], rx: i32, ry: i32) -> Result<Self> {
        let mut r = Reader::new(data);
        let blank = Tile { height: 0, overlay: 0, shape: 0, rotation: 0, settings: 0, underlay: 0 };
        let mut tiles = vec![blank; PLANES * SIZE * SIZE];
        for plane in 0..PLANES {
            for x in 0..SIZE {
                for y in 0..SIZE {
                    let below = if plane > 0 { tiles[((plane - 1) * SIZE + x) * SIZE + y].height } else { 0 };
                    let t = &mut tiles[(plane * SIZE + x) * SIZE + y];
                    let (ax, ay) = (rx * 64 + x as i32, ry * 64 + y as i32);
                    loop {
                        let op = if r.remaining() > 0 { r.u8()? } else { 0 };
                        match op {
                            0 => {
                                t.height = if plane == 0 { (-noise_height(ax + 932731, ay + 556238) * 8) << 2 } else { below - 960 };
                                break;
                            }
                            1 => {
                                let mut h = r.u8()? as i32;
                                if h == 1 {
                                    h = 0;
                                }
                                t.height = if plane == 0 { (-h * 8) << 2 } else { below - ((h * 8) << 2) };
                                break;
                            }
                            2..=49 => {
                                t.overlay = r.u8()?;
                                t.shape = (op - 2) / 4;
                                t.rotation = (op - 2) & 3;
                            }
                            50..=81 => t.settings = op - 49,
                            _ => t.underlay = op - 81,
                        }
                    }
                }
            }
        }
        Ok(Terrain { tiles })
    }
}

/// `Trig.COSINE[i]`: 16384 × cos(i × 2π / 16384), truncated.
fn cosine(i: i32) -> i32 {
    (16384.0 * (i as f64 * 3.834951969714103E-4).cos()) as i32
}

/// `Class260.method4632`: a pseudo-random byte for (x, y).
fn random(x: i32, y: i32) -> i32 {
    let mut n = y.wrapping_mul(57).wrapping_add(x);
    n ^= n.wrapping_shl(13);
    let m = n.wrapping_mul(n.wrapping_mul(n).wrapping_mul(15731).wrapping_add(789221)).wrapping_add(1376312589) & 0x7fff_ffff;
    (m >> 19) & 0xff
}

/// `Class241.method4151`: `random` smoothed over its neighbours.
fn smooth(x: i32, y: i32) -> i32 {
    let corners = random(x - 1, y - 1) + random(x + 1, y - 1) + random(x - 1, y + 1) + random(x + 1, y + 1);
    let sides = random(x - 1, y) + random(x + 1, y) + random(x, y - 1) + random(x, y + 1);
    corners / 16 + sides / 8 + random(x, y) / 4
}

/// `Class430.method7220`: cosine interpolation from `a` to `b` at `at` of `scale`.
fn interpolate(a: i32, b: i32, at: i32, scale: i32) -> i32 {
    let f = (65536 - cosine(at * 8192 / scale)) >> 1;
    (((65536 - f) * a) >> 16) + ((f * b) >> 16)
}

/// `Class159.method2735`: smoothed noise at (x, y) on a grid of `scale`.
fn noise(x: i32, y: i32, scale: i32) -> i32 {
    let (gx, fx, gy, fy) = (x / scale, x & (scale - 1), y / scale, y & (scale - 1));
    let top = interpolate(smooth(gx, gy), smooth(gx + 1, gy), fx, scale);
    let bottom = interpolate(smooth(gx, gy + 1), smooth(gx + 1, gy + 1), fx, scale);
    interpolate(top, bottom, fy, scale)
}

/// `CutsceneObject.calculateTileHeights`: 10..=60.
pub fn noise_height(x: i32, y: i32) -> i32 {
    let n = noise(x + 45365, y + 91923, 4) - 128 + ((noise(x + 10294, y + 37821, 2) - 128) >> 1) + ((noise(x, y, 1) - 128) >> 2);
    ((n as f64 * 0.3) as i32 + 35).clamp(10, 60)
}

/// `MapRegion.getArchiveName`: `m` (terrain) or `l` (locations), region x and y.
pub fn archive_name(terrain: bool, rx: i32, ry: i32) -> String {
    format!("{}{rx}_{ry}", if terrain { 'm' } else { 'l' })
}

/// `AbstractRenderer.method8696`: the name hash a reference table keys archives by
/// (`h × 31 + byte`, on the lower-case name).
pub fn name_hash(name: &str) -> i32 {
    name.to_lowercase().bytes().fold(0i32, |h, b| h.wrapping_mul(31).wrapping_add(b as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_read_their_opcodes() {
        // Plane 0, tile (0, 0): overlay 7 shape 0 rotation 1 (op 3), settings 1 (op 50),
        // underlay 4 (op 85), height 5 (op 1, 5); every other tile ends at once (op 0).
        let mut data = vec![3, 7, 50, 85, 1, 5];
        data.extend(std::iter::repeat_n(0u8, PLANES * SIZE * SIZE - 1));
        let t = Terrain::decode(&data, 50, 50).unwrap();
        let first = t.tile(0, 0, 0);
        assert_eq!((first.overlay, first.shape, first.rotation, first.settings, first.underlay, first.height), (7, 0, 1, 1, 4, -160));
        // A plane above with no height is 960 above the one below.
        assert_eq!(t.tile(1, 0, 0).height, -160 - 960);
        assert!((10..=60).contains(&noise_height(3200, 3200)));
    }

    #[test]
    fn names_hash_like_java() {
        // Java's `"m50_50".hashCode()` and `"l50_50".hashCode()`.
        assert_eq!((name_hash("m50_50"), name_hash("L50_50")), (-1123920270, -1152549421));
        assert_eq!(archive_name(false, 50, 49), "l50_49");
    }
}
