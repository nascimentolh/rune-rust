//! 3D models (index 7, file 0 of each archive). Port of `RSMesh(byte[])`: `decodeNewFormat`
//! (files ending in two 0xff bytes) and `decodeOldFormat`.
//!
//! Kept: vertices, triangles, face colours, face types and alphas, the version, each face's
//! texture and texture mapping, and the mappings (`decodeTexturedTriangles`). Read past
//! (their own sections, located but not decoded): priorities, skins, particle emitters and
//! billboards.

use super::buffer::Reader;
use super::error::{corrupt, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    /// `vertexX/Y/Z`.
    pub vertices: Vec<[i32; 3]>,
    /// `triangleX/Y/Z`: vertex indices of each face.
    pub faces: Vec<[u16; 3]>,
    /// `faceColor`: 16-bit HSL.
    pub colours: Vec<u16>,
    /// `faceType`: 0 smooth, 1 flat, 2 not drawn...; empty when the model has none (all 0).
    pub face_types: Vec<i8>,
    /// `faceAlphas`: 0 opaque..255; empty when the model has none (all 0).
    pub alphas: Vec<u8>,
    /// `version`: 12 unless the new format's flag 8 gives it (byte 24 from the end).
    pub version: u8,
    /// `faceTextures`: each face's texture, −1 for none; empty when the model has none.
    pub textures: Vec<i16>,
    /// `texturePos`: each face's mapping (an index into `mappings`), −1 to map the texture on
    /// its own corners; empty when the model has none.
    pub texture_pos: Vec<i8>,
    /// `textureRenderTypes`, `texTriX/Y/Z` and the values that go with them.
    pub mappings: Vec<Mapping>,
}

/// How a texture lies on the faces that use it (`decodeTexturedTriangles`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mapping {
    /// `textureRenderTypes`: 0 the plane through three vertices; 1, 2 and 3 projected about
    /// those vertices with the values below (another kind reads nothing).
    pub kind: i8,
    /// `texTriX/Y/Z`: the three vertices.
    pub corners: [u16; 3],
    /// `particleDirectionX/Y/Z` (so named by the deobfuscator): the projection's scale, kinds
    /// 1..=3.
    pub scale: [i32; 3],
    /// `particleLifespanX/Y/Z`, kinds 1..=3.
    pub params: [i32; 3],
    /// `texturePrimaryColor`, `textureSecondaryColor`, kind 2.
    pub colours: [i32; 2],
}

/// Reads `count` bytes at `pos`, or nothing when the section isn't there.
fn section(data: &[u8], present: bool, pos: usize, count: usize) -> Result<Vec<u8>> {
    if !present {
        return Ok(Vec::new());
    }
    Ok(at(data, pos).bytes(count)?.to_vec())
}

/// `ByteBuf.readUnsignedSmart` (signed, despite the name): one byte − 64 or two bytes − 49152.
fn smart(r: &mut Reader) -> Result<i32> {
    if r.peek_u8()? < 128 {
        Ok(r.u8()? as i32 - 64)
    } else {
        Ok(r.u16()? as i32 - 49152)
    }
}

fn at<'a>(data: &'a [u8], pos: usize) -> Reader<'a> {
    let mut r = Reader::new(data);
    r.seek(pos);
    r
}

/// The vertex positions: flags at `flags`, then x, y, z deltas in their own sections.
fn vertices(data: &[u8], count: usize, flags: usize, xyz: [usize; 3]) -> Result<Vec<[i32; 3]>> {
    let mut f = at(data, flags);
    let mut axes = xyz.map(|p| at(data, p));
    let mut last = [0i32; 3];
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let bits = f.u8()?;
        for (k, r) in axes.iter_mut().enumerate() {
            if bits & (1 << k) != 0 {
                last[k] += smart(r)?;
            }
        }
        out.push(last);
    }
    Ok(out)
}

/// `calculateMaxDepth`: the faces, from strip codes at `codes` and index deltas at `indices`.
/// Java's `short` arithmetic is kept (`as i16`).
fn faces(data: &[u8], count: usize, indices: usize, codes: usize) -> Result<Vec<[u16; 3]>> {
    let (mut idx, mut code) = (at(data, indices), at(data, codes));
    let (mut a, mut b, mut c, mut last) = (0i16, 0i16, 0i16, 0i16);
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        match code.u8()? {
            1 => {
                a = (smart(&mut idx)? + last as i32) as i16;
                b = (smart(&mut idx)? + a as i32) as i16;
                c = (smart(&mut idx)? + b as i32) as i16;
                last = c;
            }
            2 => {
                b = c;
                c = (smart(&mut idx)? + last as i32) as i16;
                last = c;
            }
            3 => {
                a = c;
                c = (smart(&mut idx)? + last as i32) as i16;
                last = c;
            }
            4 => {
                std::mem::swap(&mut a, &mut b);
                c = (smart(&mut idx)? + last as i32) as i16;
                last = c;
            }
            // Other codes leave the face as it was (0, 0, 0 for a new array).
            _ => {
                out.push([0; 3]);
                continue;
            }
        }
        out.push([a as u16, b as u16, c as u16]);
    }
    Ok(out)
}

/// `decodeTexturedTriangles`: each mapping by its kind from the sections after the vertices:
/// kind 0 three vertices (u16) from the first; kinds 1..=3 three vertices from the second, the
/// scale from the third (u16, u24 for y from version 14, all u24 from 15), one byte each from
/// the fourth and fifth and one from the sixth, where a kind 2 mapping then has its two colour
/// bytes.
fn mappings(data: &[u8], kinds: &[i8], version: u8, at_: usize) -> Result<Vec<Mapping>> {
    let simple = kinds.iter().filter(|&&k| k == 0).count();
    let projected = kinds.iter().filter(|&&k| (1..=3).contains(&k)).count();
    let scale_size = match version {
        14 => 7,
        v if v >= 15 => 9,
        _ => 6,
    };
    let mut first = at(data, at_);
    let mut second = at(data, at_ + simple * 6);
    let scales_at = at_ + simple * 6 + projected * 6;
    let mut third = at(data, scales_at);
    let mut fourth = at(data, scales_at + projected * scale_size);
    let mut fifth = at(data, scales_at + projected * scale_size + projected);
    let mut sixth = at(data, scales_at + projected * scale_size + projected * 2);
    kinds
        .iter()
        .map(|&kind| {
            let mut m = Mapping { kind, ..Mapping::default() };
            if kind == 0 {
                m.corners = [first.u16()?, first.u16()?, first.u16()?];
            }
            if (1..=3).contains(&kind) {
                m.corners = [second.u16()?, second.u16()?, second.u16()?];
                m.scale = if version < 15 {
                    let x = third.u16()? as i32;
                    let y = if version < 14 { third.u16()? as i32 } else { third.u24()? as i32 };
                    [x, y, third.u16()? as i32]
                } else {
                    [third.u24()? as i32, third.u24()? as i32, third.u24()? as i32]
                };
                m.params = [fourth.i8()? as i32, fifth.i8()? as i32, sixth.i8()? as i32];
            }
            if kind == 2 {
                m.colours = [sixth.i8()? as i32, sixth.i8()? as i32];
            }
            Ok(m)
        })
        .collect()
}

fn colours(data: &[u8], count: usize, pos: usize) -> Result<Vec<u16>> {
    let mut r = at(data, pos);
    (0..count).map(|_| r.u16()).collect()
}

impl Mesh {
    /// `upscale`: positions × 4 (object models before version 13).
    /// With the mappings' scales: × 4 for the first as many mappings as there are projected
    /// ones (the reference's array is that long, whatever their kinds), z only when the
    /// mapping isn't kind 1.
    pub fn upscale(&mut self) {
        for v in &mut self.vertices {
            for c in v {
                *c <<= 2;
            }
        }
        let projected = self.mappings.iter().filter(|m| (1..=3).contains(&m.kind)).count();
        for m in self.mappings.iter_mut().take(projected) {
            m.scale[0] <<= 2;
            m.scale[1] <<= 2;
            if m.kind != 1 {
                m.scale[2] <<= 2;
            }
        }
    }

    /// `RSMesh(RSMesh[], n)`: `parts` as one mesh, faces in order; corners at the same place
    /// share one vertex (`method2657`), vertices no face uses are left out. Face types and alphas
    /// are kept if any part has them (0 for the others), textures and mappings too (−1 for the
    /// others; mapping indices moved past the earlier parts' mappings). Then each part's
    /// mappings: a kind 0 mapping's vertices found the same way; the other kinds keep their
    /// part's vertex numbers, as the reference does.
    pub fn merge(parts: &[Mesh]) -> Mesh {
        let types = parts.iter().any(|m| !m.face_types.is_empty());
        let alphas = parts.iter().any(|m| !m.alphas.is_empty());
        let textures = parts.iter().any(|m| !m.textures.is_empty());
        let texture_pos = parts.iter().any(|m| !m.texture_pos.is_empty());
        let mut out = Mesh {
            vertices: Vec::new(),
            faces: Vec::new(),
            colours: Vec::new(),
            face_types: Vec::new(),
            alphas: Vec::new(),
            version: 12,
            textures: Vec::new(),
            texture_pos: Vec::new(),
            mappings: Vec::new(),
        };
        let corner = |out: &mut Mesh, m: &Mesh, v: u16| {
            let p = m.vertices[v as usize];
            match out.vertices.iter().position(|&q| q == p) {
                Some(k) => k as u16,
                None => {
                    out.vertices.push(p);
                    (out.vertices.len() - 1) as u16
                }
            }
        };
        for m in parts {
            for (i, face) in m.faces.iter().enumerate() {
                if types {
                    out.face_types.push(m.face_types.get(i).copied().unwrap_or(0));
                }
                if alphas {
                    out.alphas.push(m.alphas.get(i).copied().unwrap_or(0));
                }
                if textures {
                    out.textures.push(m.textures.get(i).copied().unwrap_or(-1));
                }
                let f = [corner(&mut out, m, face[0]), corner(&mut out, m, face[1]), corner(&mut out, m, face[2])];
                out.faces.push(f);
                out.colours.push(m.colours[i]);
            }
        }
        for m in parts {
            let before = out.mappings.len() as i32;
            if texture_pos {
                for i in 0..m.faces.len() {
                    let pos = m.texture_pos.get(i).copied().unwrap_or(-1);
                    out.texture_pos.push(if pos != -1 { (before + pos as i32) as i8 } else { -1 });
                }
            }
            for t in &m.mappings {
                let mut t = t.clone();
                if t.kind == 0 {
                    t.corners = t.corners.map(|v| corner(&mut out, m, v));
                }
                out.mappings.push(t);
            }
        }
        out
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 23 {
            return corrupt("mesh too short");
        }
        if data[data.len() - 1] == 0xff && data[data.len() - 2] == 0xff {
            Mesh::new_format(data)
        } else {
            Mesh::old_format(data)
        }
    }

    /// `decodeNewFormat`: a 23-byte header at the end, then each section's start is the sum
    /// of the sizes before it.
    fn new_format(data: &[u8]) -> Result<Self> {
        let mut h = at(data, data.len() - 23);
        let (vertex_count, face_count) = (h.u16()? as usize, h.u16()? as usize);
        let textured = h.u8()? as usize;
        let flags = h.u8()?;
        let face_types = flags & 1 == 1;
        let version = if flags & 8 == 8 { data[data.len() - 24] } else { 12 };
        let priority = h.u8()?;
        let (alpha, face_skins, textures, vertex_skins) = (h.u8()? == 1, h.u8()? == 1, h.u8()? == 1, h.u8()? == 1);
        let (len_x, len_y, len_z) = (h.u16()? as usize, h.u16()? as usize, h.u16()? as usize);
        let (face_indices, texture_indices) = (h.u16()? as usize, h.u16()? as usize);
        let flags_at = textured;
        let mut pos = flags_at + vertex_count;
        if face_types {
            pos += face_count;
        }
        let strips_at = pos;
        pos += face_count;
        for (present, n) in [(priority == 255, face_count), (face_skins, face_count), (vertex_skins, vertex_count)] {
            if present {
                pos += n;
            }
        }
        let alphas_at = pos;
        if alpha {
            pos += face_count;
        }
        let indices_at = pos;
        pos += face_indices;
        let textures_at = pos;
        if textures {
            pos += face_count * 2;
        }
        let texture_pos_at = pos;
        let colours_at = pos + texture_indices;
        let x_at = colours_at + face_count * 2;
        let (y_at, z_at) = (x_at + len_x, x_at + len_x + len_y);
        // The mappings' kinds are the file's first bytes.
        let kinds: Vec<i8> = section(data, true, 0, textured)?.into_iter().map(|b| b as i8).collect();
        let face_textures: Vec<i16> = if textures {
            let mut r = at(data, textures_at);
            (0..face_count).map(|_| Ok((r.u16()? as i32 - 1) as i16)).collect::<Result<_>>()?
        } else {
            Vec::new()
        };
        // Each textured face's mapping, one byte − 1 (only when the model has mappings).
        let texture_pos = if textures && textured > 0 {
            let mut r = at(data, texture_pos_at);
            face_textures.iter().map(|&t| Ok(if t != -1 { (r.u8()? as i32 - 1) as i8 } else { -1 })).collect::<Result<_>>()?
        } else {
            Vec::new()
        };
        Ok(Mesh {
            vertices: vertices(data, vertex_count, flags_at, [x_at, y_at, z_at])?,
            faces: faces(data, face_count, indices_at, strips_at)?,
            colours: colours(data, face_count, colours_at)?,
            face_types: section(data, face_types, flags_at + vertex_count, face_count)?.into_iter().map(|b| b as i8).collect(),
            alphas: section(data, alpha, alphas_at, face_count)?,
            version,
            textures: face_textures,
            texture_pos,
            mappings: mappings(data, &kinds, version, z_at + len_z)?,
        })
    }

    /// `decodeOldFormat`.
    fn old_format(data: &[u8]) -> Result<Self> {
        let mut h = at(data, data.len() - 18);
        let (vertex_count, face_count) = (h.u16()? as usize, h.u16()? as usize);
        let textured = h.u8()? as usize;
        let (render_types, priority, alpha, face_skins, vertex_skins) = (h.u8()?, h.u8()?, h.u8()?, h.u8()?, h.u8()?);
        let (len_x, len_y) = (h.u16()? as usize, h.u16()? as usize);
        h.u16()?; // z length
        let face_indices = h.u16()? as usize;
        let mut pos = vertex_count;
        let strips_at = pos;
        pos += face_count;
        if priority == 255 {
            pos += face_count;
        }
        if face_skins == 1 {
            pos += face_count;
        }
        if render_types == 1 {
            pos += face_count;
        }
        if vertex_skins == 1 {
            pos += vertex_count;
        }
        let alphas_at = pos;
        if alpha == 1 {
            pos += face_count;
        }
        let indices_at = pos;
        pos += face_indices;
        let colours_at = pos;
        let mappings_at = colours_at + face_count * 2;
        pos += face_count * 2 + textured * 6;
        let (x_at, y_at) = (pos, pos + len_x);
        let z_at = y_at + len_y;
        let vertices = vertices(data, vertex_count, 0, [x_at, y_at, z_at])?;
        // Render type bit 1 makes a face flat (type 1, else 0); with bit 2 it is textured: its
        // colour field held the texture, the colour becomes 127, and the type's other bits
        // (>> 2) are its mapping.
        let mut colours = colours(data, face_count, colours_at)?;
        let types_at = vertex_count + face_count + usize::from(priority == 255) * face_count + usize::from(face_skins == 1) * face_count;
        let render = section(data, render_types == 1, types_at, face_count)?;
        let (mut textures, mut texture_pos) = (Vec::new(), Vec::new());
        for (c, t) in colours.iter_mut().zip(&render) {
            if t & 2 == 2 {
                textures.push(*c as i16);
                texture_pos.push((t >> 2) as i8);
                *c = 127;
            } else {
                textures.push(-1);
                texture_pos.push(-1);
            }
        }
        // Old models' mappings are all kind 0.
        let mut r = at(data, mappings_at);
        let mappings: Vec<Mapping> =
            (0..textured).map(|_| Ok(Mapping { corners: [r.u16()?, r.u16()?, r.u16()?], ..Mapping::default() })).collect::<Result<_>>()?;
        let faces = faces(data, face_count, indices_at, strips_at)?;
        // A face mapped on its own three corners needs no mapping; with none left, none are kept.
        let mut kept = false;
        for (pos, face) in texture_pos.iter_mut().zip(&faces) {
            if *pos != -1 {
                let Some(m) = mappings.get(*pos as u8 as usize) else { return corrupt("mesh texture mapping out of range") };
                if m.corners == *face {
                    *pos = -1;
                } else {
                    kept = true;
                }
            }
        }
        if !kept {
            texture_pos.clear();
        }
        if textures.iter().all(|&t| t == -1) {
            textures.clear();
        }
        Ok(Mesh {
            vertices,
            faces,
            colours,
            face_types: render.iter().map(|t| (t & 1) as i8).collect(),
            alphas: section(data, alpha == 1, alphas_at, face_count)?,
            version: 12,
            textures,
            texture_pos,
            mappings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mappings_read_each_kind_from_its_own_section() {
        // Kinds [0, 2] at version 14: one plane (3 u16), then one projected mapping's vertices
        // (3 u16), its scale (u16, u24, u16), one byte each from the next two sections, and from
        // the last its byte and then its two colours.
        let mut d = vec![0, 1, 0, 2, 0, 3];
        d.extend([0, 4, 0, 5, 0, 6]);
        d.extend([0, 7, 0, 0, 8, 0, 9]);
        d.extend([10u8, 11, 12, 13, 14]);
        let m = mappings(&d, &[0, 2], 14, 0).unwrap();
        assert_eq!(m[0], Mapping { kind: 0, corners: [1, 2, 3], ..Mapping::default() });
        assert_eq!(m[1], Mapping { kind: 2, corners: [4, 5, 6], scale: [7, 8, 9], params: [10, 11, 12], colours: [13, 14] });
    }
}
