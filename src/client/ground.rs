//! The ground of the scene around the player: plane 0 of the 104 × 104 tiles `REGION` names
//! (`MapRegion.loadMapSceneNormal`), cut into triangles and lit as the OpenGL toolkit does it.
//!
//! - Colours (`Class329.method5837`): the underlay blended over the 11 × 11 tiles around it
//!   (running sums, packed by `Class371.method6348`); an overlay keeps its own colour.
//! - Shapes (`Class329.method5845`, ground blending off): each tile's overlay and underlay
//!   cover parts of it (half tiles, corners, diagonals) cut from 13 points, turned by the
//!   tile's rotation; a tile with neither isn't drawn.
//! - Light (`OpenGLGround`'s constructor and `SA`): a normal at every tile corner from the
//!   heights around it, blended across the tile; each point's colour is its lightness × 74 / 128
//!   through the colour table, then scaled by the scene light (`Light`).
//!
//! Not yet: ground blending (`method5846`, the default preference: the edges between floors are
//! blended), textures (water...), scenery shadows (`Ground.LA`, counted as none), the planes
//! above, and locations.

use super::interface::assets::Assets;
use crate::cache::floors::{pack_hsl, Overlay, Underlay};
use crate::cache::map_terrain::{Terrain, SIZE};
use macroquad::prelude::*;
use std::collections::HashMap;

/// `MapSize.SIZE_104`: tiles across the scene.
pub const SCENE: usize = 104;

/// `Class329.anIntArray3811` / `anIntArray3800`: the 13 points a tile's shapes are cut from
/// (x, y inside the tile, 512 across).
const POINT_X: [i32; 13] = [0, 256, 512, 512, 512, 256, 0, 0, 128, 256, 128, 384, 256];
const POINT_Y: [i32; 13] = [0, 0, 0, 256, 512, 512, 512, 256, 256, 384, 128, 128, 256];
/// `OVERLAY_FACE_COUNT` / `UNDERLAY_FACE_COUNT`: triangles each shape gives the overlay and
/// the underlay.
const OVERLAY_FACES: [usize; 15] = [2, 1, 1, 1, 2, 2, 2, 1, 3, 3, 3, 2, 0, 4, 0];
const UNDERLAY_FACES: [usize; 15] = [0, 1, 2, 2, 1, 1, 2, 3, 1, 3, 3, 4, 2, 0, 4];
/// `anIntArrayArray3824`, `anIntArrayArray3860`, `anIntArrayArray3815`: each shape's triangles
/// as three of the 13 points, the overlay's first.
const FACE_A: [&[usize]; 15] = [
    &[0, 2],
    &[0, 2],
    &[0, 0, 2],
    &[2, 0, 0],
    &[0, 2, 0],
    &[0, 0, 2],
    &[0, 5, 1, 4],
    &[0, 4, 4, 4],
    &[4, 4, 4, 0],
    &[6, 6, 6, 2, 2, 2],
    &[2, 2, 2, 6, 6, 6],
    &[0, 11, 6, 6, 6, 4],
    &[0, 2],
    &[0, 4, 4, 4],
    &[0, 4, 4, 4],
];
const FACE_B: [&[usize]; 15] = [
    &[2, 4],
    &[2, 4],
    &[5, 2, 4],
    &[4, 5, 2],
    &[2, 4, 5],
    &[5, 2, 4],
    &[1, 6, 2, 5],
    &[1, 6, 7, 1],
    &[6, 7, 1, 1],
    &[0, 8, 9, 8, 9, 4],
    &[8, 9, 4, 0, 8, 9],
    &[2, 10, 0, 10, 11, 11],
    &[2, 4],
    &[1, 6, 7, 1],
    &[1, 6, 7, 1],
];
const FACE_C: [&[usize]; 15] = [
    &[6, 6],
    &[6, 6],
    &[6, 5, 5],
    &[5, 6, 5],
    &[5, 5, 6],
    &[6, 5, 5],
    &[5, 0, 4, 1],
    &[7, 7, 1, 2],
    &[7, 1, 2, 7],
    &[8, 9, 4, 0, 8, 9],
    &[0, 8, 9, 8, 9, 4],
    &[11, 0, 10, 11, 4, 2],
    &[6, 6],
    &[7, 7, 1, 2],
    &[7, 7, 1, 2],
];

/// A triangle of a tile: three points inside it (0..=512 each way) and its packed HSL colour
/// (-1: not drawn).
type Face = ([(i32, i32); 3], i32);

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

    /// How bright a point facing `normal` is. `SA` works it out itself when light detail is off
    /// (`flags & 7 == 0`); with it on, OpenGL's two lights (the sun, and -1.2 from the other
    /// side) over the ambient give the same.
    fn strength(&self, normal: [f32; 3]) -> f32 {
        let d = self.sun[0] * normal[0] + self.sun[1] * normal[1] + self.sun[2] * normal[2];
        self.ambient + d * if d > 0.0 { 0.69921875 } else { 1.2 }
    }
}

pub struct Ground {
    /// Absolute tile of the scene's (0, 0) corner (`coordGrid`).
    pub base: (i32, i32),
    /// `tileHeights[0]`, (SCENE + 1)² corners.
    heights: Vec<i32>,
    /// The lit triangles, in meshes small enough for one macroquad draw call each.
    meshes: Vec<Mesh>,
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
    /// table's).
    pub fn load(assets: &mut Assets, chunk: (i32, i32), brightness: f64, light: &Light) -> Self {
        let (list, base) = regions(chunk.0, chunk.1);
        let n = SCENE;
        let mut heights = vec![0; (n + 1) * (n + 1)];
        let mut floors = vec![(0u8, 0u8, 0u8, 0u8); n * n];
        for (rx, ry) in list {
            let Some(t) = assets.terrain(rx, ry) else { continue };
            place(&t, (rx * 64 - base.0, ry * 64 - base.1), &mut heights, &mut floors);
        }
        let mut underlays: HashMap<u8, Option<Underlay>> = HashMap::new();
        let mut overlays: HashMap<u8, Option<Overlay>> = HashMap::new();
        let mut underlay = |id: u8, assets: &mut Assets| match id {
            0 => None,
            id => underlays.entry(id).or_insert_with(|| assets.underlay(id as u32 - 1)).clone(),
        };
        let underlay_defs: Vec<Option<Underlay>> = floors.iter().map(|f| underlay(f.0, assets)).collect();
        let blended = blend(n, |x, y| underlay_defs[x * n + y].clone());
        let mut ground = Ground { base, heights, meshes: Vec::new() };
        let mut faces = Vec::with_capacity(n * n);
        for (i, &(_, o, shape, rotation)) in floors.iter().enumerate() {
            let overlay = match o {
                0 => None,
                id => overlays.entry(id).or_insert_with(|| assets.overlay(id as u32 - 1)).clone(),
            };
            // `pathShape` 0 with no overlay is the whole tile underlay (shape 12).
            let shape = if shape == 0 && overlay.is_none() { 12 } else { shape };
            // An overlay with neither colour is left out (its shape still cuts the underlay).
            let overlay = overlay.filter(|d| d.colour != -1 || d.secondary != -1).map(|d| d.colour);
            // `ints_4[x][y]`: 0 where the blend wrote nothing.
            let underlay = underlay_defs[i].as_ref().map(|_| blended[i].unwrap_or(0));
            faces.push(tile_faces(shape, rotation, overlay, underlay));
        }
        ground.meshes = ground.light(&faces, brightness, light);
        ground
    }

    /// `OpenGLGround`'s constructor (normals) and `SA` (each point's colour), for every tile's
    /// `faces`, in meshes of at most `MESH` points.
    fn light(&self, faces: &[Vec<Face>], brightness: f64, light: &Light) -> Vec<Mesh> {
        const MESH: usize = 4998;
        let normals = self.normals();
        let mut meshes = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();
        for x in 0..SCENE {
            for y in 0..SCENE {
                for &(points, hsl) in &faces[x * SCENE + y] {
                    if hsl == -1 {
                        continue;
                    }
                    if vertices.len() + 3 > MESH {
                        meshes.push(mesh(std::mem::take(&mut vertices)));
                    }
                    for (ox, oy) in points {
                        let (fx, fy) = ((x << 9) as i32 + ox, (y << 9) as i32 + oy);
                        let strength = light.strength(normal_at(&normals, x, y, ox, oy));
                        let colour = lit(hsl, strength, brightness);
                        let h = self.average_height(fx, fy);
                        vertices.push(Vertex::new(-(fx as f32), -(h as f32), fy as f32, 0.0, 0.0, colour));
                    }
                }
            }
        }
        if !vertices.is_empty() {
            meshes.push(mesh(vertices));
        }
        meshes
    }

    /// `OpenGLGround`'s constructor: the normal at each corner inside the scene, from the
    /// heights either side ((dx, -1024, dy) made length 1); 0 on the edges.
    fn normals(&self) -> Vec<[f32; 3]> {
        let m = SCENE + 1;
        let mut out = vec![[0.0; 3]; m * m];
        for y in 1..SCENE {
            for x in 1..SCENE {
                let dx = self.corner_height(x + 1, y) - self.corner_height(x - 1, y);
                let dy = self.corner_height(x, y + 1) - self.corner_height(x, y - 1);
                let f = (1.0 / ((512 * 512 * 4 + dx * dx + dy * dy) as f64).sqrt()) as f32;
                out[x * m + y] = [dx as f32 * f, (-512 * 2) as f32 * f, dy as f32 * f];
            }
        }
        out
    }

    /// `Ground.getHeight`: the height at tile corner (`x`, `y`).
    pub fn corner_height(&self, x: usize, y: usize) -> i32 {
        self.heights[x * (SCENE + 1) + y]
    }

    /// `Ground.averageHeight`: the height at fine position (`x`, `y`), blended between the
    /// tile's four corners (0 off the scene).
    pub fn average_height(&self, x: i32, y: i32) -> i32 {
        let (tx, ty) = (x >> 9, y >> 9);
        let n = SCENE as i32;
        if tx < 0 || ty < 0 || tx > n - 1 || ty > n - 1 {
            return 0;
        }
        let (fx, fy) = (x & 511, y & 511);
        let (tx, ty) = (tx as usize, ty as usize);
        let south = (self.corner_height(tx, ty) * (512 - fx) + fx * self.corner_height(tx + 1, ty)) >> 9;
        let north = (self.corner_height(tx, ty + 1) * (512 - fx) + self.corner_height(tx + 1, ty + 1) * fx) >> 9;
        (north * fy + south * (512 - fy)) >> 9
    }

    /// The fine position (512 a tile, tile centres at + 256) of absolute tile (`x`, `y`).
    pub fn fine(&self, x: i32, y: i32) -> (i32, i32) {
        ((x - self.base.0) * 512 + 256, (y - self.base.1) * 512 + 256)
    }

    /// A flat ground with nothing drawn (for tests).
    #[cfg(test)]
    pub fn flat() -> Self {
        Ground { base: (0, 0), heights: vec![0; (SCENE + 1) * (SCENE + 1)], meshes: Vec::new() }
    }

    /// Draws the ground through the game camera's `view`, in a viewport `size` pixels.
    /// East (+x) is drawn as −x: looking north with y up, macroquad's +x is on the left.
    pub fn draw(&self, view: &super::camera::View, size: (f32, f32)) {
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
        for m in &self.meshes {
            draw_mesh(m);
        }
        set_default_camera();
    }
}

/// A mesh of separate triangles.
fn mesh(vertices: Vec<Vertex>) -> Mesh {
    let indices = (0..vertices.len() as u16).collect();
    Mesh { vertices, indices, texture: None }
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
    let light = ((74 * (hsl & 0x7f)) >> 7).clamp(2, 126);
    let rgb = super::palette::hsl_to_rgb(((hsl & 0xff80) | light) as u16, brightness);
    let channel = |c: i32| ((c & 0xff) as f32 * strength) as i32;
    let (r, g, b) = (channel(rgb >> 16), channel(rgb >> 8), channel(rgb));
    Color::from_rgba(r.clamp(0, 255) as u8, g.clamp(0, 255) as u8, b.clamp(0, 255) as u8, 255)
}

/// `Class329.method5845` for one tile of shape `shape` (12: all underlay) turned `rotation`
/// quarters: the overlay's triangles in its colour, then the underlay's in the blended one.
fn tile_faces(shape: u8, rotation: u8, overlay: Option<i32>, underlay: Option<i32>) -> Vec<Face> {
    let s = shape as usize;
    if overlay.is_none() && underlay.is_none() {
        return Vec::new();
    }
    let point = |i: usize| {
        let (x, y) = (POINT_X[i], POINT_Y[i]);
        match rotation {
            1 => (y, 512 - x),
            2 => (512 - x, 512 - y),
            3 => (512 - y, x),
            _ => (x, y),
        }
    };
    let face = |k: usize, colour: i32| ([point(FACE_A[s][k]), point(FACE_B[s][k]), point(FACE_C[s][k])], colour);
    let mut out = Vec::new();
    if let Some(colour) = overlay {
        out.extend((0..OVERLAY_FACES[s]).map(|k| face(k, colour)));
    }
    if let Some(colour) = underlay {
        out.extend((OVERLAY_FACES[s]..OVERLAY_FACES[s] + UNDERLAY_FACES[s]).map(|k| face(k, colour)));
    }
    out
}

/// Copies a region's plane 0 into the scene at `offset` (tiles outside are dropped).
fn place(t: &Terrain, offset: (i32, i32), heights: &mut [i32], floors: &mut [Floor]) {
    let n = SCENE as i32;
    for x in 0..SIZE as i32 {
        for y in 0..SIZE as i32 {
            let (sx, sy) = (x + offset.0, y + offset.1);
            if sx < 0 || sy < 0 || sx >= n || sy >= n {
                continue;
            }
            let tile = t.tile(0, x as usize, y as usize);
            heights[(sx * (n + 1) + sy) as usize] = tile.height;
            floors[(sx * n + sy) as usize] = (tile.underlay, tile.overlay, tile.shape, tile.rotation);
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
    fn shapes_split_a_tile_between_overlay_and_underlay() {
        // Shape 12 (no overlay): two underlay triangles over the whole tile.
        let all = tile_faces(12, 0, None, Some(7));
        assert_eq!(all, vec![([(0, 0), (512, 0), (0, 512)], 7), ([(512, 0), (512, 512), (0, 512)], 7)]);
        // Shape 1, a diagonal half: one overlay triangle, one underlay; turned once, (x, y)
        // becomes (y, 512 − x).
        let half = tile_faces(1, 1, Some(3), Some(7));
        assert_eq!(half, vec![([(0, 512), (0, 0), (512, 512)], 3), ([(0, 0), (512, 0), (512, 512)], 7)]);
        // Neither floor: nothing drawn.
        assert!(tile_faces(0, 0, None, None).is_empty());
    }

    #[test]
    fn flat_ground_faces_the_sun() {
        let light = Light { ambient: 1.0, sun: [0.0, -1.0, 0.0] };
        let g = Ground::flat();
        let n = g.normals();
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
