//! What is drawn of the scene, and the roofs hidden over the player (`SceneObjectManager`).
//!
//! Every ground tile and object is a `Piece` with the tile slots it stands on. A tile's slot is
//! its plane, one lower on bridge tiles (`tileMasks[1] & 2`: `SceneObjectManager.method3382`
//! moves each slot down; an object anchored there drops a plane). The draw hides a piece on slot
//! p ≥ player plane + 1 where the roof map marks it this frame (`method3424` for objects, every
//! tile of a scenery object's span marked; `method3441` for ground tiles).
//!
//! The roof map (`removeRoofsOptionOverride`, set from the remove-roofs preference when the game
//! starts, `Class190`): 0 marks nothing; 1 floods every roof tile once (`Class17.method570`); 2,
//! each frame (`QuickchatMessageLoader.method7215`): from the player's tile when it is under a
//! roof (`tileMasks & 4`), else, with the camera pitch below 2560, from the first roof tile on the
//! line from the camera's tile to the player's. The flood (`JS5Manager.method5492`) spreads over
//! the player plane's roof tiles and the tiles bordering them, marking the planes above (unless
//! `tileMasks & 8`); on a border tile a plane is left alone when its wall or roof edge faces the
//! way the flood came (so outer walls and eaves stay).
//!
//! Not yet: the marked roofs' bounds (`anIntArray7198`...), occlusion culling (`Class201`), the
//! tile under a bridge (`aClass293_3509`).

use super::ground::SCENE;
use crate::cache::map_terrain::PLANES;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::{
    draw_mesh, gl_use_default_material, gl_use_material, load_material, Material, MaterialParams, Mesh, ShaderSource, Texture2D, Vertex,
};
use macroquad::prelude::{Comparison, PipelineParams, UniformDesc, UniformType};
use std::cell::OnceCell;
use std::collections::HashMap;

/// Where a piece stands, for hiding it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Spot {
    /// A ground tile, wall, decoration: one tile on a slot.
    Tile { slot: usize, x: usize, y: usize },
    /// A scenery object or roof: every tile it covers (`localX`, `aShort9458`...).
    Span { slot: usize, x0: usize, x1: usize, y0: usize, y1: usize },
}

pub struct Piece {
    pub spot: Spot,
    pub vertices: Vec<Vertex>,
    /// Each triangle's texture, −1 for none.
    pub textures: Vec<i16>,
    /// A ground tile: drawn before every object (`OpenGLGround` draws the whole ground first).
    pub ground: bool,
}

/// One tile slot's walls and scenery, as the flood checks them.
#[derive(Clone, Debug, Default)]
pub struct Slot {
    /// The tile's two wall slots: their orientation codes (`aShort9615`).
    pub walls: [u8; 2],
    /// Scenery on the tile: (type | rotation << 6, with type 21 as 19) and the span it covers.
    pub scenery: Vec<(u8, [usize; 4])>,
}

/// Every tile slot, `[slot][x][y]`.
pub struct Slots(Vec<Slot>);

impl Default for Slots {
    fn default() -> Self {
        Slots(vec![Slot::default(); PLANES * SCENE * SCENE])
    }
}

impl Slots {
    pub fn at(&mut self, slot: usize, x: usize, y: usize) -> &mut Slot {
        &mut self.0[(slot * SCENE + x) * SCENE + y]
    }

    fn get(&self, slot: usize, x: usize, y: usize) -> &Slot {
        &self.0[(slot * SCENE + x) * SCENE + y]
    }
}

/// `WallObjectGraphNode.anIntArray10522` / `anIntArray10518` via `Class235.method3967`: a wall
/// piece's orientation code (corners, types 1 and 3, use the second table).
pub fn wall_orientation(kind: u8, rotation: i32) -> u8 {
    let r = (rotation & 3) as usize;
    if kind == 1 || kind == 3 {
        [16, 32, 64, 128][r]
    } else {
        [1, 2, 4, 8][r]
    }
}

/// `Class125.method2172`: the wall orientation a roof edge code (type | rotation << 6) faces.
fn edge_orientation(code: u8) -> u8 {
    let (kind, r) = (code & 0x3f, (code >> 6) & 3);
    match kind {
        18 => [1, 2, 4, 8][r as usize],
        19 | 21 => [16, 32, 64, 128][r as usize],
        _ => 0,
    }
}

/// The scene's tile settings, `[plane][x][y]` (`regionMap.tileMasks`).
pub type Settings<'a> = &'a [Vec<u8>];

fn mask(settings: Settings, plane: usize, x: usize, y: usize) -> u8 {
    settings[plane][x * SCENE + y]
}

/// The roof map: tiles marked `[slot][x][y]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Roofs(Vec<bool>);

impl Roofs {
    fn none() -> Self {
        Roofs(vec![false; PLANES * SCENE * SCENE])
    }

    fn marked(&self, slot: usize, x: usize, y: usize) -> bool {
        self.0[(slot * SCENE + x) * SCENE + y]
    }

    fn mark(&mut self, slot: usize, x: usize, y: usize) {
        self.0[(slot * SCENE + x) * SCENE + y] = true;
    }

    /// `JS5Manager.method5492` from tile (`x`, `y`) on the player's `plane`: false when the tile
    /// is already marked or isn't a roof tile.
    fn flood(&mut self, settings: Settings, slots: &Slots, plane: usize, x: usize, y: usize) -> bool {
        if self.marked(plane, x, y) || mask(settings, plane, x, y) & 4 == 0 {
            return false;
        }
        let n = SCENE;
        // (x, y, and the edge codes the flood came across).
        let mut queue = std::collections::VecDeque::from([(x, y, [0u8; 3])]);
        self.mark(plane, x, y);
        while let Some((x, y, codes)) = queue.pop_front() {
            let border = mask(settings, plane, x, y) & 4 == 0;
            for above in plane + 1..PLANES {
                if mask(settings, above, x, y) & 8 != 0 {
                    continue;
                }
                let slot = slots.get(above, x, y);
                if border {
                    let faces = |o: u8| o != 0 && (slot.walls[0] == o || slot.walls[1] == o);
                    if slot.walls[0] != 0
                        && (faces(edge_orientation(codes[0]))
                            || (codes[1] != 0 && faces(edge_orientation(codes[1])))
                            || (codes[2] != 0 && faces(edge_orientation(codes[2]))))
                    {
                        continue;
                    }
                    if slot
                        .scenery
                        .iter()
                        .any(|&(c, _)| c == codes[0] || (codes[1] != 0 && c == codes[1]) || (codes[2] != 0 && c == codes[2]))
                    {
                        continue;
                    }
                }
                for &(_, [x0, x1, y0, y1]) in &slot.scenery {
                    if x0 != x1 || y0 != y1 {
                        for sx in x0..=x1 {
                            for sy in y0..=y1 {
                                self.mark(above, sx, sy);
                            }
                        }
                    }
                }
                self.mark(above, x, y);
            }
            if border {
                continue;
            }
            let roof = |x: usize, y: usize| mask(settings, plane, x, y) & 4 != 0;
            let mut push = |s: &mut Self, nx: usize, ny: usize, codes: [u8; 3]| {
                queue.push_back((nx, ny, codes));
                s.mark(plane, nx, ny);
            };
            if x >= 1 && !self.marked(plane, x - 1, y) {
                push(self, x - 1, y, [0x12, 0xd3, 0x13]);
            }
            if y + 1 < n {
                if x >= 1 && !self.marked(plane, x - 1, y + 1) && !roof(x, y + 1) && !roof(x - 1, y) {
                    push(self, x - 1, y + 1, [0x12, 0x52, 0x13]);
                }
                if !self.marked(plane, x, y + 1) {
                    push(self, x, y + 1, [0x52, 0x13, 0x53]);
                }
                if x + 1 < n && !self.marked(plane, x + 1, y + 1) && !roof(x, y + 1) && !roof(x + 1, y) {
                    push(self, x + 1, y + 1, [0x52, 0x92, 0x53]);
                }
            }
            if x + 1 < n && !self.marked(plane, x + 1, y) {
                push(self, x + 1, y, [0x92, 0x53, 0x93]);
            }
            if y >= 1 {
                if x >= 1 && !self.marked(plane, x - 1, y - 1) && !roof(x, y - 1) && !roof(x - 1, y) {
                    push(self, x - 1, y - 1, [0xd2, 0x12, 0xd3]);
                }
                if !self.marked(plane, x, y - 1) {
                    push(self, x, y - 1, [0xd2, 0x93, 0xd3]);
                }
                if x + 1 < n && !self.marked(plane, x + 1, y - 1) && !roof(x, y - 1) && !roof(x + 1, y) {
                    push(self, x + 1, y - 1, [0x92, 0xd2, 0x93]);
                }
            }
        }
        true
    }

    /// The roof map for `mode` with the player on `plane` at `player` (tile) and the camera at
    /// `camera` (tile) with `pitch`.
    pub fn compute(
        mode: i32,
        settings: Settings,
        slots: &Slots,
        plane: usize,
        player: (usize, usize),
        camera: (i32, i32),
        pitch: i32,
    ) -> Option<Roofs> {
        let mut roofs = Roofs::none();
        match mode {
            0 => return None,
            // `Class17.method570`: every roof tile, at most 512 floods.
            1 => {
                let mut floods = 0;
                for x in 0..SCENE {
                    for y in 0..SCENE {
                        if roofs.flood(settings, slots, plane, x, y) {
                            floods += 1;
                        }
                        if floods >= 512 {
                            return Some(roofs);
                        }
                    }
                }
            }
            _ => {
                if plane == 3 {
                    return Some(roofs);
                }
                let (px, py) = player;
                if mask(settings, plane, px, py) & 4 != 0 {
                    roofs.flood(settings, slots, plane, px, py);
                } else if pitch < 2560 {
                    roofs.line(settings, slots, plane, camera, (px as i32, py as i32));
                }
            }
        }
        Some(roofs)
    }

    /// `method7215`'s walk from the camera's tile towards the player's, flooding from the first
    /// roof tile on the way.
    fn line(&mut self, settings: Settings, slots: &Slots, plane: usize, from: (i32, i32), to: (i32, i32)) {
        let n = SCENE as i32;
        let (mut x, mut y) = from;
        let (dx, dy) = ((to.0 - x).abs(), (to.1 - y).abs());
        if (dx == 0 && dy == 0) || dx >= n || dy >= n {
            return;
        }
        let step = |v: &mut i32, target: i32| {
            if *v < target {
                *v += 1;
            } else if *v > target {
                *v -= 1;
            }
        };
        let hit = |s: &mut Self, x: i32, y: i32| {
            if !(0..n).contains(&x) || !(0..n).contains(&y) {
                return false;
            }
            if mask(settings, plane, x as usize, y as usize) & 4 != 0 {
                s.flood(settings, slots, plane, x as usize, y as usize);
                return true;
            }
            false
        };
        let mut frac = 32768;
        if dx > dy {
            let slope = dy * 65536 / dx;
            while x != to.0 {
                step(&mut x, to.0);
                if hit(self, x, y) {
                    return;
                }
                frac += slope;
                if frac >= 65536 {
                    frac -= 65536;
                    step(&mut y, to.1);
                    if hit(self, x, y) {
                        return;
                    }
                }
            }
        } else {
            let slope = dx * 65536 / dy;
            while y != to.1 {
                step(&mut y, to.1);
                if hit(self, x, y) {
                    return;
                }
                frac += slope;
                if frac >= 65536 {
                    frac -= 65536;
                    step(&mut x, to.0);
                    if hit(self, x, y) {
                        return;
                    }
                }
            }
        }
    }
}

/// The pieces of the scene and the meshes of those shown.
#[derive(Default)]
pub struct Scene {
    pieces: Vec<Piece>,
    /// Which pieces the meshes hold.
    shown: Vec<bool>,
    /// Each mesh with its texture id (−1 plain).
    meshes: Vec<(i16, Mesh)>,
    /// The generated textures by id (a texture that couldn't be made draws plain).
    textures: HashMap<i16, SceneTexture>,
    /// Draws textured triangles with the texture repeating (its coordinates run past 0..1).
    repeat: OnceCell<Option<Material>>,
}

/// A texture made for the card, with its slide speed (`textureSpeedU`, `textureSpeedV`).
pub struct SceneTexture {
    pub image: Texture2D,
    pub speed: [i8; 2],
}

/// `OpenGLRenderer.method13609`: a sliding texture's coordinates move by
/// ((t mod 128000) / 1000) × speed / 64 mod 1 at clock `t` (`Utils.time()` as an int, set each
/// frame by `method8398`), in floats; the ground's scale divides first (`Node_Sub3`).
fn slide(clock: i32, speed: [i8; 2]) -> [f32; 2] {
    let seconds = (clock % 128000) as f32 / 1000.0;
    speed.map(|s| seconds * s as f32 / 64.0 % 1.0)
}

/// The default 3D material with the texture coordinates moved by `Offset` and wrapped
/// (`fract`): the reference's textures repeat, which macroquad offers no safe way to ask the
/// texture for.
const REPEAT_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

/// `Offset`: the slide (`slide`).
const REPEAT_FRAGMENT: &str = r#"#version 100
varying lowp vec4 color;
varying highp vec2 uv;
uniform sampler2D Texture;
uniform highp vec2 Offset;
void main() {
    gl_FragColor = color * texture2D(Texture, fract(uv + Offset));
}"#;

fn repeat_material() -> Option<Material> {
    let pipeline_params = PipelineParams {
        depth_test: Comparison::LessOrEqual,
        depth_write: true,
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    let uniforms = vec![UniformDesc::new("Offset", UniformType::Float2)];
    load_material(
        ShaderSource::Glsl { vertex: REPEAT_VERTEX, fragment: REPEAT_FRAGMENT },
        MaterialParams { pipeline_params, uniforms, ..Default::default() },
    )
    .map_err(|e| eprintln!("texture material: {e}"))
    .ok()
}

impl Scene {
    pub fn new(pieces: Vec<Piece>, textures: HashMap<i16, SceneTexture>) -> Self {
        Scene { pieces, shown: Vec::new(), meshes: Vec::new(), textures, repeat: OnceCell::new() }
    }

    /// The textures the pieces use.
    pub fn texture_ids(pieces: &[Piece]) -> Vec<i16> {
        let mut ids: Vec<i16> = pieces.iter().flat_map(|p| p.textures.iter().copied()).filter(|&t| t != -1).collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Hides the pieces `roofs` covers above the player's `plane` (`method3424`, `method3441`),
    /// rebuilding the meshes when that changes.
    pub fn show(&mut self, roofs: Option<&Roofs>, plane: usize) {
        let limit = plane + 1;
        let shown: Vec<bool> = self
            .pieces
            .iter()
            .map(|p| {
                let Some(r) = roofs else { return true };
                match p.spot {
                    Spot::Tile { slot, x, y } => slot < limit || !r.marked(slot, x, y),
                    Spot::Span { slot, x0, x1, y0, y1 } => {
                        slot < limit || (x0..=x1.min(SCENE - 1)).any(|x| (y0..=y1.min(SCENE - 1)).any(|y| !r.marked(slot, x, y)))
                    }
                }
            })
            .collect();
        if shown == self.shown {
            return;
        }
        // One run of meshes per texture, each mesh at most 4998 vertices, drawn plain first and
        // then by texture: the ground's later materials fade over the earlier ones.
        let mut groups: std::collections::BTreeMap<(bool, i16), Vec<Vertex>> = std::collections::BTreeMap::new();
        for (p, _) in self.pieces.iter().zip(&shown).filter(|(_, &s)| s) {
            for (tri, &t) in p.vertices.chunks(3).zip(&p.textures) {
                let key = if self.textures.contains_key(&t) { t } else { -1 };
                groups.entry((!p.ground, key)).or_default().extend_from_slice(tri);
            }
        }
        let mut meshes = Vec::new();
        for ((_, key), vertices) in groups {
            let texture = self.textures.get(&key).map(|t| t.image.clone());
            for part in vertices.chunks(4998) {
                meshes.push((key, mesh(part.to_vec(), texture.clone())));
            }
        }
        self.meshes = meshes;
        self.shown = shown;
    }

    /// How many textures were made for the card, and how many of them slide.
    pub fn texture_count(&self) -> (usize, usize) {
        (self.textures.len(), self.textures.values().filter(|t| t.speed != [0; 2]).count())
    }

    /// How many pieces are hidden now.
    pub fn hidden(&self) -> usize {
        self.shown.iter().filter(|&&s| !s).count()
    }

    pub fn draw(&self) {
        let repeat = self.repeat.get_or_init(repeat_material);
        // `Utils.time()` cast to an int, as `client` passes it to `method8398`.
        let clock = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64 as i32);
        for (id, m) in &self.meshes {
            match (&m.texture, repeat) {
                (Some(_), Some(material)) => {
                    let speed = self.textures.get(id).map_or([0; 2], |t| t.speed);
                    let [u, v] = slide(clock, speed);
                    material.set_uniform("Offset", (u, v));
                    gl_use_material(material);
                    draw_mesh(m);
                    gl_use_default_material();
                }
                _ => draw_mesh(m),
            }
        }
    }
}

/// A mesh of separate triangles.
fn mesh(vertices: Vec<Vertex>, texture: Option<Texture2D>) -> Mesh {
    let indices = (0..vertices.len() as u16).collect();
    Mesh { vertices, indices, texture }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_roof_hides_the_plane_above_but_keeps_outer_edges() {
        // Plane 0 has a 3 × 3 roof block at 10..=12; plane 1 above it.
        let mut settings = vec![vec![0u8; SCENE * SCENE]; PLANES];
        for x in 10..=12 {
            for y in 10..=12 {
                settings[0][x * SCENE + y] = 4;
            }
        }
        let mut slots = Slots::default();
        // An outside wall on the border tile west of the block (9, 11), facing east (orientation 4,
        // rotation 2), is what a flood coming from the east (code 0x12: 18 rotation 0 → 1)
        // doesn't match, so it is marked; one facing west (1) is left alone.
        slots.at(1, 9, 11).walls = [1, 0];
        let roofs = Roofs::compute(2, &settings, &slots, 0, (11, 11), (11, 2), 1081).unwrap();
        assert!(roofs.marked(1, 11, 11) && roofs.marked(1, 13, 11));
        assert!(!roofs.marked(1, 9, 11));
        assert!(!roofs.marked(1, 20, 20));
        // A piece on slot 1 there is hidden, one on slot 0 isn't.
        let mut scene = Scene::new(
            vec![
                Piece {
                    spot: Spot::Tile { slot: 1, x: 11, y: 11 },
                    vertices: vec![Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, macroquad::prelude::WHITE); 3],
                    textures: vec![-1],
                    ground: true,
                },
                Piece {
                    spot: Spot::Tile { slot: 0, x: 11, y: 11 },
                    vertices: vec![Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, macroquad::prelude::WHITE); 3],
                    textures: vec![5],
                    ground: false,
                },
            ],
            HashMap::new(),
        );
        scene.show(Some(&roofs), 0);
        assert_eq!(scene.shown, vec![false, true]);
    }

    #[test]
    fn sliding_textures_move_by_the_clock() {
        // 2.5 s at speed 64: one turn and a half, so half way; speed −32: a quarter back.
        assert_eq!(slide(2500, [64, -32]), [0.5, -0.25]);
        // The clock wraps every 128 s.
        assert_eq!(slide(128_000 + 500, [64, 0]), [0.5, 0.0]);
    }

    #[test]
    fn walls_and_edges_have_orientations() {
        assert_eq!((wall_orientation(0, 2), wall_orientation(3, 1), wall_orientation(2, 6)), (4, 32, 4));
        assert_eq!((edge_orientation(0x12), edge_orientation(0xd3), edge_orientation(5)), (1, 128, 0));
    }
}
