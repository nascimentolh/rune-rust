//! The ground of the scene around the player: the four planes of the 104 × 104 tiles `REGION`
//! names (`MapRegion.loadMapSceneNormal`), cut into triangles and lit as the OpenGL toolkit does
//! it, plane by plane (`Class329.method5837`).
//!
//! - Colours (`method5837`): the underlay blended over the 11 × 11 tiles around it (running
//!   sums, packed by `Class371.method6348`); an overlay keeps its own colour. The blended colours
//!   are one table for all planes: a plane keeps the plane below's where it blends nothing.
//! - Shapes (`tiles.rs`): each tile cut into overlay and underlay triangles, blended between
//!   floors when the ground blending preference is on (`method5846`) or not (`method5845`).
//! - Light (`OpenGLGround`'s constructor and `SA`): a normal at every tile corner from the
//!   heights around it on its plane, blended across the tile; each point's colour is its
//!   lightness × 74 / 128 through the colour table, then scaled by the scene light (`Light`).
//!
//! Each lit tile is a piece for the scene (`scene.rs`), which hides the planes above the player
//! under roofs. Not yet: textures (water...) and scenery shadows (`Ground.LA`, counted as none).

use super::interface::assets::Assets;
use super::scene::{Piece, Scene, Spot};
use super::tiles::{self, Face, Floors};
use crate::cache::floors::{pack_hsl, Overlay, Underlay};
use crate::cache::map_terrain::{Terrain, PLANES, SIZE};
use macroquad::prelude::*;
use std::collections::HashMap;

/// `MapSize.SIZE_104`: tiles across the scene.
pub const SCENE: usize = 104;

/// The scene's light: `Class239.method4049` with `Atmosphere`'s defaults, given to the toolkit
/// by `OpenGLRenderer.IA` (ambient) and `m` (the sun: colour white, strengths 0.69921875 lit
/// and 1.2 shaded, direction (-50, -60, -50) × 4).
pub struct Light {
    ambient: f32,
    /// `aFloatArray8426`: towards the sun, length 1 (x, height, y).
    sun: [f32; 3],
}

impl Light {
    /// For the brightness preference `brightness`; each map load adds -0.05..0.05 at random
    /// (`MapRegion.aFloat3173`).
    pub fn scene(brightness: i32) -> Self {
        let region = (-0.05 + ::rand::random::<f64>() / 10.0) as f32;
        let ambient = (0.7 + brightness as f32 * 0.1 + region) * 1.1523438;
        let (x, y, z) = ((-50 << 2) as f32, (-60 << 2) as f32, (-50 << 2) as f32);
        let f = (1.0 / ((x * x + y * y + z * z) as f64).sqrt()) as f32;
        Light { ambient, sun: [x * f, y * f, z * f] }
    }

    /// A model corner's brightness (`MeshRasterizer_Sub1`'s light loop): its normal sum `n`
    /// over `count` faces × 256 (a flat face's own, count 0, over 384), and the sun's strengths
    /// × 768 / `contrast`.
    pub fn model_strength(&self, n: [i16; 3], count: i8, contrast: i32) -> f32 {
        let dot = self.sun[0] * n[0] as f32 + self.sun[1] * n[1] as f32 + self.sun[2] * n[2] as f32;
        let d = if count == 0 { dot * 0.002_604_166_7 } else { dot / (count as i32 * 256) as f32 };
        let (lit, shaded) = (0.69921875 * 768.0 / contrast as f32, 1.2 * 768.0 / contrast as f32);
        self.ambient + d * if d < 0.0 { shaded } else { lit }
    }

    /// How bright a ground point facing `normal` is. `SA` works it out itself when light detail
    /// is off (`flags & 7 == 0`); with it on, OpenGL's two lights (the sun, and -1.2 from the
    /// other side) over the ambient give the same.
    fn strength(&self, normal: [f32; 3]) -> f32 {
        let d = self.sun[0] * normal[0] + self.sun[1] * normal[1] + self.sun[2] * normal[2];
        self.ambient + d * if d > 0.0 { 0.69921875 } else { 1.2 }
    }
}

pub struct Ground {
    /// Absolute tile of the scene's (0, 0) corner (`coordGrid`).
    pub base: (i32, i32),
    /// `tileHeights`, per plane (SCENE + 1)² corners.
    heights: Vec<Vec<i32>>,
    /// `regionMap.tileMasks`, per plane SCENE² tiles.
    settings: Vec<Vec<u8>>,
    /// The lit tiles, one piece each, for the scene to take.
    pub pieces: Vec<Piece>,
}

/// The regions a scene centred on chunk (`cx`, `cy`) covers, and its base tile
/// (`loadMapSceneNormal`: chunks cx ± 6, as regions of 8 chunks).
pub fn regions(cx: i32, cy: i32) -> (Vec<(i32, i32)>, (i32, i32)) {
    let half = (SCENE >> 4) as i32;
    let list = ((cx - half) / 8..=(cx + half) / 8).flat_map(|rx| ((cy - half) / 8..=(cy + half) / 8).map(move |ry| (rx, ry))).collect();
    (list, ((cx - half) * 8, (cy - half) * 8))
}

/// One tile's floors as `place` copies them: (underlay, overlay, shape, rotation).
type Floor = (u8, u8, u8, u8);

impl Ground {
    /// Loads, shapes and lights the scene around chunk `chunk` (`brightness` is the colour
    /// table's; `blending` the ground blending preference).
    pub fn load(assets: &mut Assets, chunk: (i32, i32), brightness: f64, light: &Light, blending: bool) -> Self {
        let (list, base) = regions(chunk.0, chunk.1);
        let n = SCENE;
        let mut heights = vec![vec![0; (n + 1) * (n + 1)]; PLANES];
        let mut floors = vec![vec![(0u8, 0u8, 0u8, 0u8); n * n]; PLANES];
        let mut settings = vec![vec![0u8; n * n]; PLANES];
        for (rx, ry) in list {
            let Some(t) = assets.terrain(rx, ry) else { continue };
            for plane in 0..PLANES {
                let at = (rx * 64 - base.0, ry * 64 - base.1);
                place(&t, plane, at, &mut heights[plane], &mut floors[plane], &mut settings[plane]);
            }
        }
        let mut underlays: HashMap<u8, Option<Underlay>> = HashMap::new();
        let mut overlays: HashMap<u8, Option<Overlay>> = HashMap::new();
        let mut ground = Ground { base, heights, settings, pieces: Vec::new() };
        // `ints_5`: one table for every plane, 0 where nothing was ever blended.
        let mut blended = vec![0; n * n];
        for (plane, floors) in floors.iter().enumerate() {
            let mut underlay_defs = Vec::with_capacity(n * n);
            let mut overlay_defs = Vec::with_capacity(n * n);
            for &(u, o, _, _) in floors {
                underlay_defs.push(match u {
                    0 => None,
                    id => underlays.entry(id).or_insert_with(|| assets.underlay(id as u32 - 1)).clone(),
                });
                overlay_defs.push(match o {
                    0 => None,
                    id => overlays.entry(id).or_insert_with(|| assets.overlay(id as u32 - 1)).clone(),
                });
            }
            for (i, c) in blend(n, |x, y| underlay_defs[x * n + y].clone()).into_iter().enumerate() {
                if let Some(c) = c {
                    blended[i] = c;
                }
            }
            let scene = Floors {
                tiles: floors,
                overlays: &overlay_defs,
                underlays: &underlay_defs,
                blended: &blended,
                heights: &ground.heights[plane],
            };
            let mut faces = Vec::with_capacity(n * n);
            for x in 0..n {
                for y in 0..n {
                    let i = x * n + y;
                    faces.push(if blending {
                        scene.blended(x, y)
                    } else {
                        let (_, _, shape, rotation) = floors[i];
                        let overlay = overlay_defs[i].as_ref();
                        // `pathShape` 0 with no overlay is the whole tile underlay (shape 12).
                        let shape = if shape == 0 && overlay.is_none() { 12 } else { shape };
                        // An overlay with neither colour is left out (its shape still cuts the underlay).
                        let colour = overlay.filter(|d| d.colour != -1 || d.secondary != -1).map(|d| d.colour);
                        tiles::plain(shape, rotation, colour, underlay_defs[i].as_ref().map(|_| blended[i]))
                    });
                }
            }
            let pieces = ground.light(plane, &faces, brightness, light);
            ground.pieces.extend(pieces);
        }
        ground
    }

    /// `OpenGLGround`'s constructor (normals) and `SA` (each point's colour), for every tile's
    /// `faces` on `plane`: one piece a tile, on the plane's slot (one lower on a bridge tile).
    fn light(&self, plane: usize, faces: &[Vec<Face>], brightness: f64, light: &Light) -> Vec<Piece> {
        let normals = self.normals(plane);
        let mut pieces = Vec::new();
        for x in 0..SCENE {
            for y in 0..SCENE {
                let mut vertices = Vec::new();
                for face in &faces[x * SCENE + y] {
                    // A triangle with no colour at all isn't drawn (`SA`: no material).
                    if face.iter().all(|p| p.2 == -1) {
                        continue;
                    }
                    for &(ox, oy, hsl) in face {
                        let (fx, fy) = ((x << 9) as i32 + ox, (y << 9) as i32 + oy);
                        let strength = light.strength(normal_at(&normals, x, y, ox, oy));
                        let colour = lit(hsl, strength, brightness);
                        let h = self.average_height(plane, fx, fy);
                        vertices.push(Vertex::new(-(fx as f32), -(h as f32), fy as f32, 0.0, 0.0, colour));
                    }
                }
                if !vertices.is_empty() {
                    let textures = vec![-1; vertices.len() / 3];
                    pieces.push(Piece { spot: Spot::Tile { slot: self.slot(plane, x, y), x, y }, vertices, textures });
                }
            }
        }
        pieces
    }

    /// The tile slot plane `plane`'s things at (`x`, `y`) end up in: one lower on a bridge tile
    /// (`tileMasks[1] & 2`, `SceneObjectManager.method3382`).
    pub fn slot(&self, plane: usize, x: usize, y: usize) -> usize {
        if plane >= 1 && self.settings[1][x * SCENE + y] & 2 != 0 {
            plane - 1
        } else {
            plane
        }
    }

    /// The tile settings, `[plane][x × SCENE + y]` (`regionMap.tileMasks`).
    pub fn settings(&self) -> &[Vec<u8>] {
        &self.settings
    }

    /// `OpenGLGround`'s constructor: the normal at each corner inside the scene, from the
    /// heights either side on `plane` ((dx, -1024, dy) made length 1); 0 on the edges.
    fn normals(&self, plane: usize) -> Vec<[f32; 3]> {
        let m = SCENE + 1;
        let mut out = vec![[0.0; 3]; m * m];
        for y in 1..SCENE {
            for x in 1..SCENE {
                let dx = self.corner_height(plane, x + 1, y) - self.corner_height(plane, x - 1, y);
                let dy = self.corner_height(plane, x, y + 1) - self.corner_height(plane, x, y - 1);
                let f = (1.0 / ((512 * 512 * 4 + dx * dx + dy * dy) as f64).sqrt()) as f32;
                out[x * m + y] = [dx as f32 * f, (-512 * 2) as f32 * f, dy as f32 * f];
            }
        }
        out
    }

    /// `Ground.getHeight`: the height at tile corner (`x`, `y`) on `plane`.
    pub fn corner_height(&self, plane: usize, x: usize, y: usize) -> i32 {
        self.heights[plane][x * (SCENE + 1) + y]
    }

    /// `Ground.averageHeight`: the height at fine position (`x`, `y`) on `plane`, blended
    /// between the tile's four corners (0 off the scene).
    pub fn average_height(&self, plane: usize, x: i32, y: i32) -> i32 {
        let (tx, ty) = (x >> 9, y >> 9);
        let n = SCENE as i32;
        if tx < 0 || ty < 0 || tx > n - 1 || ty > n - 1 {
            return 0;
        }
        let (fx, fy) = (x & 511, y & 511);
        let (tx, ty) = (tx as usize, ty as usize);
        let h = |cx: usize, cy: usize| self.corner_height(plane, cx, cy);
        let south = (h(tx, ty) * (512 - fx) + fx * h(tx + 1, ty)) >> 9;
        let north = (h(tx, ty + 1) * (512 - fx) + h(tx + 1, ty + 1) * fx) >> 9;
        (north * fy + south * (512 - fy)) >> 9
    }

    /// The fine position (512 a tile, tile centres at + 256) of absolute tile (`x`, `y`).
    pub fn fine(&self, x: i32, y: i32) -> (i32, i32) {
        ((x - self.base.0) * 512 + 256, (y - self.base.1) * 512 + 256)
    }

    /// A flat ground with nothing drawn (for tests).
    #[cfg(test)]
    pub fn flat() -> Self {
        let (heights, settings) = (vec![vec![0; (SCENE + 1) * (SCENE + 1)]; PLANES], vec![vec![0; SCENE * SCENE]; PLANES]);
        Ground { base: (0, 0), heights, settings, pieces: Vec::new() }
    }
}

/// Draws `scene` through the game camera's `view`, in a viewport `size` pixels. East (+x) is
/// drawn as −x: looking north with y up, macroquad's +x is on the left.
pub fn draw(view: &super::camera::View, size: (f32, f32), scene: &Scene) {
    let at = |p: [i32; 3]| vec3(-(p[0] as f32), -(p[1] as f32), p[2] as f32);
    set_camera(&Camera3D {
        position: at(view.eye),
        target: at(view.focus),
        up: vec3(0.0, 1.0, 0.0),
        fovy: 2.0 * ((size.1 / 2.0) / view.focal.max(1) as f32).atan(),
        aspect: Some(size.0 / size.1),
        z_near: 50.0,
        // The whole loaded scene (the reference's draw distance isn't ported yet).
        z_far: (SCENE * 512 * 2) as f32,
        ..Default::default()
    });
    clear_background(BLACK);
    scene.draw();
    set_default_camera();
}

/// `SA`: the normal at point (`ox`, `oy`) of tile (`x`, `y`): a corner's own, else blended
/// across the tile from its four corners.
fn normal_at(normals: &[[f32; 3]], x: usize, y: usize, ox: i32, oy: i32) -> [f32; 3] {
    let m = SCENE + 1;
    let at = |cx: usize, cy: usize| normals[cx * m + cy];
    match (ox, oy) {
        (0, 0) => at(x, y),
        (0, 512) => at(x, y + 1),
        (512, 512) => at(x + 1, y + 1),
        (512, 0) => at(x + 1, y),
        _ => {
            let (fx, fy) = (ox as f32 / 512.0, oy as f32 / 512.0);
            let (sw, nw, ne, se) = (at(x, y), at(x, y + 1), at(x + 1, y + 1), at(x + 1, y));
            let mut out = [0.0; 3];
            for k in 0..3 {
                let south = sw[k] + (se[k] - sw[k]) * fx;
                let north = nw[k] + (ne[k] - nw[k]) * fx;
                out[k] = south + (north - south) * fy;
            }
            out
        }
    }
}

/// `SA` with no scenery shadow (74 − 0): lightness × 74 / 128 (2..=126) through the colour
/// table, then each channel × `strength` (`Node_Sub3.method12083`).
fn lit(hsl: i32, strength: f32, brightness: f64) -> Color {
    // A point with no colour of its own (−1) in a drawn triangle is black.
    if hsl == -1 {
        return Color::from_rgba(0, 0, 0, 255);
    }
    let light = ((74 * (hsl & 0x7f)) >> 7).clamp(2, 126);
    let rgb = super::palette::hsl_to_rgb(((hsl & 0xff80) | light) as u16, brightness);
    let channel = |c: i32| ((c & 0xff) as f32 * strength) as i32;
    let (r, g, b) = (channel(rgb >> 16), channel(rgb >> 8), channel(rgb));
    Color::from_rgba(r.clamp(0, 255) as u8, g.clamp(0, 255) as u8, b.clamp(0, 255) as u8, 255)
}

/// Copies a region's `plane` into the scene at `offset` (tiles outside are dropped).
fn place(t: &Terrain, plane: usize, offset: (i32, i32), heights: &mut [i32], floors: &mut [Floor], settings: &mut [u8]) {
    let n = SCENE as i32;
    for x in 0..SIZE as i32 {
        for y in 0..SIZE as i32 {
            let (sx, sy) = (x + offset.0, y + offset.1);
            if sx < 0 || sy < 0 || sx >= n || sy >= n {
                continue;
            }
            let tile = t.tile(plane, x as usize, y as usize);
            heights[(sx * (n + 1) + sy) as usize] = tile.height;
            floors[(sx * n + sy) as usize] = (tile.underlay, tile.overlay, tile.shape, tile.rotation);
            settings[(sx * n + sy) as usize] = tile.settings;
        }
    }
}

/// `Class329.method5837`'s underlay blend: for each tile, the sums over the 11 × 11 tiles
/// around it of hue × chroma, saturation, lightness, chroma and count, packed as
/// `method6348(hue × 256 / chroma, saturation / count, lightness / count)`; `None` where no
/// tile around has an underlay.
fn blend(n: usize, mut floor: impl FnMut(usize, usize) -> Option<Underlay>) -> Vec<Option<i32>> {
    let mut defs = vec![None; n * n];
    for x in 0..n {
        for y in 0..n {
            defs[x * n + y] = floor(x, y);
        }
    }
    let mut out = vec![None; n * n];
    let (mut hue, mut sat, mut light, mut chroma, mut count) = (vec![0; n], vec![0; n], vec![0; n], vec![0; n], vec![0; n]);
    for x in -5..n as i32 {
        for y in 0..n {
            let add = x + 5;
            if (add as usize) < n {
                if let Some(u) = &defs[add as usize * n + y] {
                    (hue[y], sat[y], light[y], chroma[y], count[y]) =
                        (hue[y] + u.hue_chroma, sat[y] + u.saturation, light[y] + u.lightness, chroma[y] + u.chroma, count[y] + 1);
                }
            }
            let remove = x - 5;
            if remove >= 0 {
                if let Some(u) = &defs[remove as usize * n + y] {
                    (hue[y], sat[y], light[y], chroma[y], count[y]) =
                        (hue[y] - u.hue_chroma, sat[y] - u.saturation, light[y] - u.lightness, chroma[y] - u.chroma, count[y] - 1);
                }
            }
        }
        if x < 0 {
            continue;
        }
        let (mut h, mut s, mut l, mut c, mut k) = (0, 0, 0, 0, 0);
        for y in -5..n as i32 {
            let add = y + 5;
            if (add as usize) < n {
                let a = add as usize;
                (h, s, l, c, k) = (h + hue[a], s + sat[a], l + light[a], c + chroma[a], k + count[a]);
            }
            let remove = y - 5;
            if remove >= 0 {
                let r = remove as usize;
                (h, s, l, c, k) = (h - hue[r], s - sat[r], l - light[r], c - chroma[r], k - count[r]);
            }
            if y >= 0 && c > 0 && k > 0 {
                out[x as usize * n + y as usize] = Some(pack_hsl(h * 256 / c, s / k, l / k));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scene_covers_the_chunks_around_its_centre() {
        // Chunk (400, 400): tiles 3152.. — regions 49..=50 both ways, base 3152.
        let (list, base) = regions(400, 400);
        assert_eq!(base, (3152, 3152));
        assert_eq!(list, vec![(49, 49), (49, 50), (50, 49), (50, 50)]);
    }

    #[test]
    fn one_underlay_everywhere_blends_to_itself() {
        let mut u = Underlay::default();
        (u.hue_chroma, u.saturation, u.lightness, u.chroma) = (100, 200, 150, 50);
        let out = blend(12, |_, _| Some(u.clone()));
        assert!(out.iter().all(|&c| c == Some(pack_hsl(100 * 256 / 50, 200, 150))));
    }

    #[test]
    fn flat_ground_faces_the_sun() {
        let light = Light { ambient: 1.0, sun: [0.0, -1.0, 0.0] };
        let g = Ground::flat();
        let n = g.normals(0);
        // Inside the scene a flat corner points straight up (height is negative up).
        assert_eq!(n[52 * (SCENE + 1) + 52], [0.0, -1.0, 0.0]);
        assert!((light.strength(n[52 * (SCENE + 1) + 52]) - (1.0 + 0.69921875)).abs() < 1e-6);
        // Lightness 127 × 74 / 128 is 73 in the colour table; strength 2 doubles each channel
        // up to 255.
        let rgb = crate::client::palette::hsl_to_rgb(73, 1.0);
        let c = |v: i32| (v & 0xff) as u8;
        assert_eq!(lit(127, 1.0, 1.0), Color::from_rgba(c(rgb >> 16), c(rgb >> 8), c(rgb), 255));
        let twice = |v: i32| ((v & 0xff) * 2).min(255) as u8;
        assert_eq!(lit(127, 2.0, 1.0), Color::from_rgba(twice(rgb >> 16), twice(rgb >> 8), twice(rgb), 255));
    }
}
