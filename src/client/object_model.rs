//! An object's model as the OpenGL toolkit builds and lights it:
//!
//! - `ObjectDefinition.method7971`: the models for the placement type, each scaled × 4 when its
//!   version is below 13, merged when several; mirrored (`inverted`, or a whole-corner wall turned
//!   past 3), turned by the rotation, recoloured, scaled and moved by the definition.
//! - `MeshRasterizer_Sub1`'s constructor: a normal at each corner, the sum of the unit (× 256)
//!   normals of the smooth faces around its vertex with their count, or a flat face's own with
//!   count 0; faces of type 2 aren't drawn.
//! - `ObjectDefinition.method8010`: a diagonal scenery piece turned another eighth; the model laid
//!   over the ground (`groundContoured` 1, 2, and 4, 5 against the plane above,
//!   `MeshRasterizer_Sub1.pa`), then nudged.
//! - The colours (`method13531`, `method13538`, the draw's light loop): lightness × (64 +
//!   ambient) / 128 through the colour table, then × the scene light with the sun scaled by
//!   768 / (850 + contrast).
//!
//! Not yet: textures (faces keep their colours), contouring 3 (`method11281`), animations,
//! billboards and particles.

use super::ground::{Ground, Light, SCENE};
use crate::cache::map_terrain::PLANES;
use crate::cache::material::TextureDetails;
use crate::cache::mesh::Mesh;
use crate::cache::object_def::ObjectDef;
use crate::trig::{cosine, sine};

/// A corner's normal as `method13542` keeps it: a short vector and a signed byte count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal {
    pub v: [i16; 3],
    pub count: i8,
}

#[derive(Clone, Debug)]
pub struct Face {
    pub corners: [usize; 3],
    pub normals: [Normal; 3],
    pub colour: u16,
    pub alpha: u8,
    /// `aShortArray8269`: the face's texture, −1 for none.
    pub texture: i16,
    /// `aFloatArray8297` / `aFloatArray8292` at each corner: where it lies on the texture.
    pub uv: [[f32; 2]; 3],
}

/// One drawn corner: where, its lit colour, its texture coordinates and its face's texture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corner {
    pub at: [i32; 3],
    pub colour: [u8; 4],
    pub uv: [f32; 2],
    pub texture: i16,
}

/// The texture table and the textures preference, as the models see them.
pub struct Textures {
    pub table: std::rc::Rc<Vec<Option<TextureDetails>>>,
    /// `textures` == 0 (`Class94.method1589`'s flag 0x40): ground textures aren't put on models.
    pub off: bool,
}

impl Textures {
    fn details(&self, id: i16) -> Option<&TextureDetails> {
        self.table.get(usize::try_from(id).ok()?)?.as_ref()
    }

    /// The texture a face is drawn with (`MeshRasterizer_Sub1`'s constructor): its own, except
    /// a ground texture with textures off.
    fn used(&self, id: i16) -> i16 {
        match self.details(id) {
            Some(d) if self.off && d.ground_mesh => -1,
            _ => id,
        }
    }
}

/// `MeshRasterizer_Sub1.method13531` for a textured face: its colour moved towards the grey of
/// `ambient` by the texture's shadow factor (of 256), then brightened by its brightness.
fn textured_colour(mut rgb: i32, ambient: i32, shadow: i32, brightness: i32) -> i32 {
    if shadow != 0 {
        let grey = if ambient < 0 {
            0
        } else if ambient > 127 {
            0xffffff
        } else {
            ambient * 131586
        };
        if shadow == 256 {
            rgb = grey;
        } else {
            let keep = 256 - shadow;
            // Java ints: the red and blue products pass 2^31 and wrap.
            let red_blue = (grey & 0xff00ff).wrapping_mul(shadow).wrapping_add((rgb & 0xff00ff).wrapping_mul(keep)) & -16711936;
            let green = (shadow * (grey & 0xff00) + keep * (rgb & 0xff00)) & 0xff0000;
            rgb = red_blue.wrapping_add(green) >> 8;
        }
    }
    if brightness != 0 {
        let b = brightness + 256;
        let r = (b * ((rgb & 0xff0000) >> 16)).min(65535);
        let g = (b * ((rgb & 0xff00) >> 8)).min(65535);
        let bl = ((rgb & 0xff) * b).min(65535);
        rgb = (bl >> 8) + (g & 0xff00) + ((r << 8) & 0xff0000);
    }
    rgb
}

/// Corners as vertices placed like the ground (east as −x, height up), with each triangle's
/// texture.
pub fn vertices(corners: &[Corner]) -> (Vec<macroquad::prelude::Vertex>, Vec<i16>) {
    let vertices = corners
        .iter()
        .map(|c| {
            let [r, g, b, a] = c.colour;
            macroquad::prelude::Vertex::new(
                -(c.at[0] as f32),
                -(c.at[1] as f32),
                c.at[2] as f32,
                c.uv[0],
                c.uv[1],
                macroquad::prelude::Color::from_rgba(r, g, b, a),
            )
        })
        .collect();
    (vertices, corners.chunks(3).map(|t| t[0].texture).collect())
}

/// `MeshRasterizer_Sub1`'s texture coordinates for a kind 0 mapping: the texture's plane
/// through `p` (its origin), `m` (u = 1) and `n` (v = 1); each corner of `face` is measured
/// along the two edges with the normal's help, in floats as the reference does.
fn planar(v: &[[i32; 3]], face: [u16; 3], [p, m, n]: [u16; 3]) -> Option<[[f32; 2]; 3]> {
    let at = |i: u16| v.get(i as usize).map(|c| c.map(|x| x as f32));
    let o = at(p)?;
    let sub = |a: [f32; 3]| [a[0] - o[0], a[1] - o[1], a[2] - o[2]];
    let (e1, e2) = (sub(at(m)?), sub(at(n)?));
    let d = [sub(at(face[0])?), sub(at(face[1])?), sub(at(face[2])?)];
    let normal = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let tu = [e2[1] * normal[2] - e2[2] * normal[1], e2[2] * normal[0] - e2[0] * normal[2], e2[0] * normal[1] - e2[1] * normal[0]];
    let iu = 1.0 / dot(tu, e1);
    let tv = [e1[1] * normal[2] - e1[2] * normal[1], e1[2] * normal[0] - e1[0] * normal[2], e1[0] * normal[1] - e1[1] * normal[0]];
    let iv = 1.0 / dot(tv, e2);
    Some(d.map(|c| [dot(tu, c) * iu, dot(tv, c) * iv]))
}

/// A face's texture and texture coordinates: none, its own corners ((0, 1), (1, 1), (0, 0)),
/// or its mapping. Projected mappings (kinds 1..=3) aren't ported yet: those faces stay plain.
fn texture_of(mesh: &Mesh, i: usize) -> (i16, [[f32; 2]; 3]) {
    let none = (-1, [[0.0; 2]; 3]);
    let texture = mesh.textures.get(i).copied().unwrap_or(-1);
    if texture == -1 {
        return none;
    }
    let pos = mesh.texture_pos.get(i).copied().unwrap_or(-1);
    if pos == -1 {
        return (texture, [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]]);
    }
    match mesh.mappings.get(pos as u8 as usize) {
        Some(m) if m.kind == 0 => planar(&mesh.vertices, mesh.faces[i], m.corners).map_or(none, |uv| (texture, uv)),
        _ => none,
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    pub vertices: Vec<[i32; 3]>,
    pub faces: Vec<Face>,
    /// `aShort8243`: 64 + the definition's ambient.
    pub ambient: i32,
    /// `aShort8257`: 850 + the definition's contrast.
    pub contrast: i32,
}

/// `MeshRasterizer_Sub1`'s face normal: the cross product, halved until each part is within
/// ±8192, then scaled to length 256.
fn face_normal(v: &[[i32; 3]], f: [u16; 3]) -> [i32; 3] {
    let at = |k: usize| v.get(f[k] as usize).copied().unwrap_or([0; 3]);
    let (a, b, c) = (at(0), at(1), at(2));
    let (dx1, dy1, dz1) = (b[0] - a[0], b[1] - a[1], b[2] - a[2]);
    let (dx2, dy2, dz2) = (c[0] - a[0], c[1] - a[1], c[2] - a[2]);
    let (mut x, mut y, mut z) = (dy1 * dz2 - dy2 * dz1, dz1 * dx2 - dz2 * dx1, dx1 * dy2 - dx2 * dy1);
    while x > 8192 || y > 8192 || z > 8192 || x < -8192 || y < -8192 || z < -8192 {
        x >>= 1;
        y >>= 1;
        z >>= 1;
    }
    let len = (((x * x + y * y + z * z) as f64).sqrt() as i32).max(1);
    [x * 256 / len, y * 256 / len, z * 256 / len]
}

impl Model {
    /// `method7971` for placement `kind` turned `rotation` (above 3: the eighth-turned or
    /// mirrored variants); `mesh` loads a model by id. `None` when the definition has no models
    /// for the type or one is missing.
    pub fn build(def: &ObjectDef, kind: i8, rotation: i32, mut mesh: impl FnMut(i32) -> Option<Mesh>) -> Option<Model> {
        let ids = def.models_for(kind)?;
        if ids.is_empty() {
            return None;
        }
        let mut parts = Vec::with_capacity(ids.len());
        for &id in ids {
            let mut m = mesh(id)?;
            if m.version < 13 {
                m.upscale();
            }
            parts.push(m);
        }
        let mesh = if parts.len() > 1 { Mesh::merge(&parts) } else { parts.remove(0) };
        let mut model = Model::lit(&mesh, 64 + def.ambient, 850 + def.contrast);
        if def.inverted || (kind == 2 && rotation > 3) {
            model.mirror();
        }
        if kind == 4 && rotation > 3 {
            model.turn(2048, true);
            model.translate([180, 0, -180]);
        }
        match rotation & 3 {
            1 => model.turn(4096, true),
            2 => model.turn(8192, true),
            3 => model.turn(12288, true),
            _ => {}
        }
        for (k, &(from, to)) in def.recolours.iter().enumerate() {
            // `aShortArray5691`, which nothing fills, stands in where a palette byte is given.
            let to = if k < def.recolour_palette.len() { 0 } else { to };
            for f in &mut model.faces {
                if f.colour == from {
                    f.colour = to;
                }
            }
        }
        if def.scale != [128; 3] {
            model.scale(def.scale);
        }
        if def.offset != [0; 3] {
            model.translate(def.offset);
        }
        // `method8010`: scenery turned past 3 is turned another eighth (positions only).
        if kind == 10 && rotation > 3 {
            model.turn(2048, false);
        }
        Some(model)
    }

    /// The constructor's corner normals and drawn faces.
    fn lit(mesh: &Mesh, ambient: i32, contrast: i32) -> Model {
        let ty = |i: usize| mesh.face_types.get(i).copied().unwrap_or(0);
        let mut sums = vec![([0i32; 3], 0i32); mesh.vertices.len()];
        let normals: Vec<[i32; 3]> = mesh.faces.iter().map(|&f| face_normal(&mesh.vertices, f)).collect();
        for (i, f) in mesh.faces.iter().enumerate() {
            if ty(i) == 0 {
                for &v in f {
                    if let Some(s) = sums.get_mut(v as usize) {
                        for (sum, add) in s.0.iter_mut().zip(normals[i]) {
                            *sum += add;
                        }
                        s.1 += 1;
                    }
                }
            }
        }
        let short = |n: [i32; 3], count: i32| Normal { v: n.map(|c| c as i16), count: count as i8 };
        let mut faces = Vec::new();
        for (i, f) in mesh.faces.iter().enumerate() {
            let corners = f.map(|v| v as usize);
            if corners.iter().any(|&v| v >= mesh.vertices.len()) {
                continue;
            }
            let normals = match ty(i) {
                0 => corners.map(|v| short(sums[v].0, sums[v].1)),
                1 => [short(normals[i], 0); 3],
                // Type 2 isn't drawn; other types get no corners (`method13542` isn't reached).
                _ => continue,
            };
            let (texture, uv) = texture_of(mesh, i);
            faces.push(Face { corners, normals, colour: mesh.colours[i], alpha: mesh.alphas.get(i).copied().unwrap_or(0), texture, uv });
        }
        Model { vertices: mesh.vertices.clone(), faces, ambient, contrast }
    }

    /// `wa`: mirrored across z (normals too).
    fn mirror(&mut self) {
        for v in &mut self.vertices {
            v[2] = -v[2];
        }
        for f in &mut self.faces {
            for n in &mut f.normals {
                n.v[2] = n.v[2].wrapping_neg();
            }
            f.corners.swap(0, 2);
            f.normals.swap(0, 2);
            f.uv.swap(0, 2);
        }
    }

    /// `S` (positions and normals) or `f` (positions only): turned `angle` of 16384 about the
    /// vertical.
    fn turn(&mut self, angle: i32, normals: bool) {
        let (s, c) = (sine(angle), cosine(angle));
        for v in &mut self.vertices {
            let x = (s * v[2] + c * v[0]) >> 14;
            v[2] = (c * v[2] - s * v[0]) >> 14;
            v[0] = x;
        }
        if normals {
            for f in &mut self.faces {
                for n in &mut f.normals {
                    let (x, z) = (n.v[0] as i32, n.v[2] as i32);
                    n.v[0] = ((z * s + x * c) >> 14) as i16;
                    n.v[2] = ((z * c - x * s) >> 14) as i16;
                }
            }
        }
    }

    /// `oa`: each axis × its scale / 128.
    fn scale(&mut self, s: [i32; 3]) {
        for v in &mut self.vertices {
            for k in 0..3 {
                if s[k] != 128 {
                    v[k] = (v[k] * s[k]) >> 7;
                }
            }
        }
    }

    /// `ia`: moved.
    fn translate(&mut self, d: [i32; 3]) {
        for v in &mut self.vertices {
            for k in 0..3 {
                v[k] += d[k];
            }
        }
    }

    /// `method13528`'s bounds as shorts: (min, max) of x, y, z.
    fn bounds(&self) -> [(i32, i32); 3] {
        let mut b = [(32767, -32768); 3];
        for v in &self.vertices {
            for k in 0..3 {
                b[k] = (b[k].0.min(v[k]), b[k].1.max(v[k]));
            }
        }
        b.map(|(lo, hi)| (lo as i16 as i32, hi as i16 as i32))
    }

    /// `method8010`'s ground fitting for a model at `at` (x, height, y) on `plane`: `pa` with the
    /// definition's mode and parameter (over the plane's ground and the one above), then its
    /// nudge.
    pub fn fit(&mut self, def: &ObjectDef, ground: &Ground, plane: usize, at: [i32; 3]) {
        if def.contoured != 0 && def.contour_param < 16384 {
            self.contour(def.contoured, def.contour_param, ground, plane, at);
        }
        if def.nudge != [0; 3] {
            self.translate(def.nudge);
        }
    }

    /// `pa` for modes 1, 2, 4 and 5 (3, `method11281`, isn't ported).
    fn contour(&mut self, mode: u8, param: i32, ground: &Ground, plane: usize, at: [i32; 3]) {
        let [(min_x, max_x), (min_y, max_y), (min_z, max_z)] = self.bounds();
        let [x, h, z] = at;
        let n = SCENE as i32;
        let (x0, x1, z0, z1) = (x + min_x, x + max_x, z + min_z, z + max_z);
        let inside = x0 >= 0 && (x1 + 512) >> 9 < n && z0 >= 0 && (z1 + 512) >> 9 < n;
        if matches!(mode, 1 | 2 | 3 | 5) && !inside {
            return;
        }
        // The ground above (`aGroundArray2607[collisionPlane + 1]`), none on the top plane.
        let upper = (plane + 1 < PLANES).then_some(plane + 1);
        if mode == 4 || mode == 5 {
            if upper.is_none() || !inside {
                return;
            }
        } else {
            let (tx0, tx1, tz0, tz1) = ((x0 >> 9) as usize, ((x1 + 511) >> 9) as usize, (z0 >> 9) as usize, ((z1 + 511) >> 9) as usize);
            if [(tx0, tz0), (tx1, tz0), (tx0, tz1), (tx1, tz1)].iter().all(|&(a, b)| ground.corner_height(plane, a, b) == h) {
                return;
            }
        }
        let height = max_y - min_y;
        match (mode, upper) {
            (1, _) => {
                for v in &mut self.vertices {
                    v[1] += ground.average_height(plane, v[0] + x, v[2] + z) - h;
                }
            }
            (2, _) => {
                if min_y == 0 {
                    return;
                }
                for v in &mut self.vertices {
                    let up = (v[1] << 16) / min_y;
                    if up < param {
                        v[1] += (ground.average_height(plane, v[0] + x, v[2] + z) - h) * (param - up) / param;
                    }
                }
            }
            (4, Some(above)) => {
                for v in &mut self.vertices {
                    v[1] += ground.average_height(above, v[0] + x, v[2] + z) - h + height;
                }
            }
            // Stretched between this ground and the one above. (A model with no height would
            // divide by 0 and throw in the reference; it is left as it is.)
            (5, Some(above)) if height != 0 => {
                for v in &mut self.vertices {
                    let (gx, gz) = (x + v[0], z + v[2]);
                    let below = ground.average_height(plane, gx, gz);
                    let span = below - ground.average_height(above, gx, gz) - param;
                    v[1] = ((span * ((v[1] << 8) / height)) >> 8) - (h - below);
                }
            }
            _ => {}
        }
    }

    /// Each face's corners at `at`, coloured and lit (`method13531`, the draw's light loop),
    /// as (x, height, y) and RGBA.
    pub fn corners(&self, at: [i32; 3], light: &Light, brightness: f64, textures: &Textures) -> Vec<Corner> {
        let mut out = Vec::with_capacity(self.faces.len() * 3);
        for f in &self.faces {
            let texture = textures.used(f.texture);
            let hsl = f.colour as i32;
            let lightness = (((hsl & 0x7f) * self.ambient) >> 7).clamp(2, 126);
            let mut rgb = super::palette::hsl_to_rgb(((hsl & 0xff80) + lightness) as u16, brightness);
            if let Some(d) = textures.details(texture) {
                rgb = textured_colour(rgb, self.ambient, d.shadow as u8 as i32, d.brightness as u8 as i32);
            }
            for k in 0..3 {
                let v = self.vertices[f.corners[k]];
                let strength = light.model_strength(f.normals[k].v, f.normals[k].count, self.contrast);
                let channel = |c: i32| (((c & 0xff) as f32 * strength) as i32).clamp(0, 255) as u8;
                out.push(Corner {
                    at: [v[0] + at[0], v[1] + at[1], v[2] + at[2]],
                    colour: [channel(rgb >> 16), channel(rgb >> 8), channel(rgb), 255 - f.alpha],
                    uv: f.uv[k],
                    texture,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One upward-facing square of two smooth faces, 128 across.
    fn square() -> Mesh {
        Mesh {
            vertices: vec![[0, 0, 0], [128, 0, 0], [128, 0, 128], [0, 0, 128]],
            faces: vec![[0, 1, 2], [0, 2, 3]],
            colours: vec![100, 100],
            face_types: Vec::new(),
            alphas: Vec::new(),
            version: 13,
            textures: Vec::new(),
            texture_pos: Vec::new(),
            mappings: Vec::new(),
        }
    }

    #[test]
    fn a_textured_face_leans_to_grey_and_brightens() {
        // No shadow or brightness: unchanged.
        assert_eq!(textured_colour(0x204060, 64, 0, 0), 0x204060);
        // Half way (128 of 256) to the grey of ambient 64 (64 × 131586 = 0x808080).
        assert_eq!(textured_colour(0x000000, 64, 128, 0), 0x404040);
        // Brightness 128: × 384 / 256.
        assert_eq!(textured_colour(0x404040, 64, 0, 128), 0x606060);
        // A shadow of 255 on white wraps past 2^31 as Java's ints do.
        assert_eq!(textured_colour(0xffffff, 64, 255, 0), 0xff80_8080_u32 as i32);
    }

    #[test]
    fn a_planar_mapping_measures_corners_along_its_edges() {
        // The texture's plane through corners 0 (origin), 1 (u) and 3 (v) of the square.
        let uv = planar(&square().vertices, [0, 1, 2], [0, 1, 3]).unwrap();
        assert_eq!(uv, [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
    }

    #[test]
    fn corner_normals_sum_their_faces() {
        let m = Model::lit(&square(), 64, 850);
        // Corner 0 is shared by both faces: two unit (256) normals, straight along y.
        assert_eq!(m.faces[0].normals[0], Normal { v: [0, -512, 0], count: 2 });
        assert_eq!(m.faces[0].normals[1], Normal { v: [0, -256, 0], count: 1 });
    }

    #[test]
    fn definitions_turn_mirror_and_scale_their_models() {
        let def = ObjectDef { models: vec![(10, vec![1])], scale: [256, 128, 128], ..ObjectDef::default() };
        let mesh = |_| Some(Mesh { version: 12, ..square() });
        // Version 12 is scaled × 4; then x × 2.
        let m = Model::build(&def, 10, 0, mesh).unwrap();
        assert_eq!(m.vertices[1], [1024, 0, 0]);
        // A quarter turn takes +x to −z.
        let m = Model::build(&def, 10, 1, mesh).unwrap();
        assert_eq!(m.vertices[1], [0, 0, -512]);
        assert!(Model::build(&def, 22, 0, mesh).is_none());
    }
}
