//! The ground of the scene around the player: plane 0 of the 104 × 104 tiles `REGION` names
//! (`MapRegion.loadMapSceneNormal`), with each tile's colour as `Class329` works it out: the
//! underlay blended over the 11 × 11 tiles around it (`method5837`'s running sums, packed by
//! `Class371.method6348`), or the overlay's own colour where one covers the tile.
//!
//! Drawn as one height-mapped grid in 3D. Not yet: overlay shapes (half tiles), textures,
//! lighting and shading (`method5845` / `method5846`), the planes above, the reference's
//! camera (a fixed stand-in view starts from `camAngleX` 1052 and zoom 600), and locations.

use super::interface::assets::Assets;
use crate::cache::floors::{pack_hsl, Underlay};
use crate::cache::map_terrain::{Terrain, SIZE};
use macroquad::prelude::*;
use std::collections::HashMap;

/// `MapSize.SIZE_104`: tiles across the scene.
pub const SCENE: usize = 104;

pub struct Ground {
    /// Absolute tile of the scene's (0, 0) corner (`coordGrid`).
    pub base: (i32, i32),
    /// `tileHeights[0]`, (SCENE + 1)² corners.
    heights: Vec<i32>,
    /// RGB per tile, -1 where no floor is drawn.
    colours: Vec<i32>,
}

/// The regions a scene centred on chunk (`cx`, `cy`) covers, and its base tile
/// (`loadMapSceneNormal`: chunks cx ± 6, as regions of 8 chunks).
pub fn regions(cx: i32, cy: i32) -> (Vec<(i32, i32)>, (i32, i32)) {
    let half = (SCENE >> 4) as i32;
    let list = ((cx - half) / 8..=(cx + half) / 8).flat_map(|rx| ((cy - half) / 8..=(cy + half) / 8).map(move |ry| (rx, ry))).collect();
    (list, ((cx - half) * 8, (cy - half) * 8))
}

impl Ground {
    /// Loads and colours the scene around chunk `chunk` (`brightness` is the colour table's).
    pub fn load(assets: &mut Assets, chunk: (i32, i32), brightness: f64) -> Self {
        let (list, base) = regions(chunk.0, chunk.1);
        let n = SCENE;
        let mut heights = vec![0; (n + 1) * (n + 1)];
        let mut underlay = vec![0u8; n * n];
        let mut overlay = vec![0u8; n * n];
        for (rx, ry) in list {
            let Some(t) = assets.terrain(rx, ry) else { continue };
            place(&t, (rx * 64 - base.0, ry * 64 - base.1), &mut heights, &mut underlay, &mut overlay);
        }
        let mut underlays: HashMap<u8, Option<Underlay>> = HashMap::new();
        let mut def = |id: u8, assets: &mut Assets| underlays.entry(id).or_insert_with(|| assets.underlay(id as u32 - 1)).clone();
        let blended = blend(n, |x, y| match underlay[x * n + y] {
            0 => None,
            id => def(id, assets),
        });
        let colours = (0..n * n)
            .map(|i| {
                let o = overlay[i];
                let overlay_colour = (o != 0).then(|| assets.overlay(o as u32 - 1)).flatten().map(|d| d.colour).filter(|&c| c >= 0);
                match (overlay_colour, blended[i]) {
                    (Some(hsl), _) | (None, Some(hsl)) => super::palette::hsl_to_rgb(hsl as u16, brightness),
                    (None, None) => -1,
                }
            })
            .collect();
        Ground { base, heights, colours }
    }

    fn height(&self, x: usize, y: usize) -> i32 {
        self.heights[x * (SCENE + 1) + y]
    }

    /// Draws the ground seen from above tile (`px`, `py`) (absolute), the stand-in view.
    pub fn draw(&self, px: i32, py: i32) {
        // East (+x) is drawn as −x: looking north with y up, macroquad's +x is on the left.
        let (lx, ly) = (-((px - self.base.0) as f32 * 512.0 + 256.0), (py - self.base.1) as f32 * 512.0 + 256.0);
        let ground_y = -(self
            .height(((px - self.base.0).clamp(0, SCENE as i32 - 1)) as usize, ((py - self.base.1).clamp(0, SCENE as i32 - 1)) as usize))
            as f32;
        // `camAngleX` 1052 of 16384 for a full turn; zoom 600 taken as tiles of 512 / 3.
        let pitch = 1052.0 / 16384.0 * std::f32::consts::TAU;
        let distance = 600.0 * 512.0 / 64.0;
        set_camera(&Camera3D {
            position: vec3(lx, ground_y + distance * pitch.sin() + 512.0, ly - distance * pitch.cos()),
            target: vec3(lx, ground_y, ly),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 2.0 * ((503.0 / 2.0) / 512.0f32).atan(),
            ..Default::default()
        });
        clear_background(BLACK);
        // Blocks of 26 × 26 tiles keep each mesh under macroquad's draw-call limits.
        const BLOCK: usize = 26;
        for bx in (0..SCENE).step_by(BLOCK) {
            for by in (0..SCENE).step_by(BLOCK) {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();
                for x in bx..(bx + BLOCK).min(SCENE) {
                    for y in by..(by + BLOCK).min(SCENE) {
                        let c = self.colours[x * SCENE + y];
                        if c < 0 {
                            continue;
                        }
                        let colour = Color::from_rgba((c >> 16) as u8, (c >> 8) as u8, c as u8, 255);
                        let at = vertices.len() as u16;
                        for (dx, dy) in [(0, 0), (1, 0), (1, 1), (0, 1)] {
                            let h = -self.height(x + dx, y + dy) as f32;
                            vertices.push(Vertex::new(-(((x + dx) * 512) as f32), h, ((y + dy) * 512) as f32, 0.0, 0.0, colour));
                        }
                        indices.extend([at, at + 1, at + 2, at, at + 2, at + 3]);
                    }
                }
                draw_mesh(&Mesh { vertices, indices, texture: None });
            }
        }
        set_default_camera();
    }
}

/// Copies a region's plane 0 into the scene at `offset` (tiles outside are dropped).
fn place(t: &Terrain, offset: (i32, i32), heights: &mut [i32], underlay: &mut [u8], overlay: &mut [u8]) {
    let n = SCENE as i32;
    for x in 0..SIZE as i32 {
        for y in 0..SIZE as i32 {
            let (sx, sy) = (x + offset.0, y + offset.1);
            if sx < 0 || sy < 0 || sx >= n || sy >= n {
                continue;
            }
            let tile = t.tile(0, x as usize, y as usize);
            let i = (sx * n + sy) as usize;
            heights[(sx * (n + 1) + sy) as usize] = tile.height;
            (underlay[i], overlay[i]) = (tile.underlay, tile.overlay);
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
}
