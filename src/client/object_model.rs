//! An object's model as the OpenGL toolkit builds and lights it:
//!
//! - `ObjectDefinition.method7971`: the models for the placement type, each scaled × 4 when its
//!   version is below 13, merged when several; mirrored (`inverted`, or a whole-corner wall turned
//!   past 3), turned by the rotation, recoloured, scaled and moved by the definition.
//! - `MeshRasterizer_Sub1`'s constructor: a normal at each corner, the sum of the unit (× 256)
//!   normals of the smooth faces around its vertex with their count, or a flat face's own with
//!   count 0; faces of type 2 aren't drawn.
//! - `ObjectDefinition.method8010`: a diagonal scenery piece turned another eighth; the model laid
//!   over the ground (`groundContoured` 1 and 2, `MeshRasterizer_Sub1.pa`), then nudged.
//! - The colours (`method13531`, `method13538`, the draw's light loop): lightness × (64 +
//!   ambient) / 128 through the colour table, then × the scene light with the sun scaled by
//!   768 / (850 + contrast).
//!
//! Not yet: textures (faces keep their colours), contouring 3 (`method11281`) and 4, 5 (they
//! need the plane above), animations, billboards and particles.

use super::ground::{Ground, Light, SCENE};
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
            faces.push(Face { corners, normals, colour: mesh.colours[i], alpha: mesh.alphas.get(i).copied().unwrap_or(0) });
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

    /// `method8010`'s ground fitting for a model at `at` (x, height, y): `pa` with the
    /// definition's mode and parameter, then its nudge.
    pub fn fit(&mut self, def: &ObjectDef, ground: &Ground, at: [i32; 3]) {
        if def.contoured != 0 && def.contour_param < 16384 {
            self.contour(def.contoured, def.contour_param, ground, at);
        }
        if def.nudge != [0; 3] {
            self.translate(def.nudge);
        }
    }

    /// `pa` for modes 1 and 2 (4 and 5 need the plane above, which isn't loaded; 3 isn't ported).
    fn contour(&mut self, mode: u8, param: i32, ground: &Ground, at: [i32; 3]) {
        let [(min_x, max_x), (min_y, _), (min_z, max_z)] = self.bounds();
        let [x, h, z] = at;
        let n = SCENE as i32;
        let (x0, x1, z0, z1) = (x + min_x, x + max_x, z + min_z, z + max_z);
        if matches!(mode, 1 | 2 | 3 | 5) && !(x0 >= 0 && (x1 + 512) >> 9 < n && z0 >= 0 && (z1 + 512) >> 9 < n) {
            return;
        }
        if mode == 4 || mode == 5 {
            return;
        }
        let (tx0, tx1, tz0, tz1) = ((x0 >> 9) as usize, ((x1 + 511) >> 9) as usize, (z0 >> 9) as usize, ((z1 + 511) >> 9) as usize);
        if [(tx0, tz0), (tx1, tz0), (tx0, tz1), (tx1, tz1)].iter().all(|&(a, b)| ground.corner_height(a, b) == h) {
            return;
        }
        match mode {
            1 => {
                for v in &mut self.vertices {
                    v[1] += ground.average_height(v[0] + x, v[2] + z) - h;
                }
            }
            2 => {
                if min_y == 0 {
                    return;
                }
                for v in &mut self.vertices {
                    let up = (v[1] << 16) / min_y;
                    if up < param {
                        v[1] += (ground.average_height(v[0] + x, v[2] + z) - h) * (param - up) / param;
                    }
                }
            }
            _ => {}
        }
    }

    /// Each face's corners at `at`, coloured and lit (`method13531`, the draw's light loop),
    /// as (x, height, y) and RGBA.
    pub fn corners(&self, at: [i32; 3], light: &Light, brightness: f64) -> Vec<([i32; 3], [u8; 4])> {
        let mut out = Vec::with_capacity(self.faces.len() * 3);
        for f in &self.faces {
            let hsl = f.colour as i32;
            let lightness = (((hsl & 0x7f) * self.ambient) >> 7).clamp(2, 126);
            let rgb = super::palette::hsl_to_rgb(((hsl & 0xff80) + lightness) as u16, brightness);
            for k in 0..3 {
                let v = self.vertices[f.corners[k]];
                let strength = light.model_strength(f.normals[k].v, f.normals[k].count, self.contrast);
                let channel = |c: i32| (((c & 0xff) as f32 * strength) as i32).clamp(0, 255) as u8;
                out.push((
                    [v[0] + at[0], v[1] + at[1], v[2] + at[2]],
                    [channel(rgb >> 16), channel(rgb >> 8), channel(rgb), 255 - f.alpha],
                ));
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
        }
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
