//! 3D models (index 7, file 0 of each archive). Port of `RSMesh(byte[])`: `decodeNewFormat`
//! (files ending in two 0xff bytes) and `decodeOldFormat`.
//!
//! Kept: vertices, triangles, face colours, face types and alphas, and the version. Read past
//! (their own sections, located but not decoded): priorities, skins, textures, texture
//! triangles, particle emitters and billboards.

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

fn colours(data: &[u8], count: usize, pos: usize) -> Result<Vec<u16>> {
    let mut r = at(data, pos);
    (0..count).map(|_| r.u16()).collect()
}

impl Mesh {
    /// `upscale`: positions × 4 (object models before version 13).
    pub fn upscale(&mut self) {
        for v in &mut self.vertices {
            for c in v {
                *c <<= 2;
            }
        }
    }

    /// `RSMesh(RSMesh[], n)`: `parts` as one mesh, faces in order; corners at the same place
    /// share one vertex (`method2657`), vertices no face uses are left out. Face types and alphas
    /// are kept if any part has them (0 for the others).
    pub fn merge(parts: &[Mesh]) -> Mesh {
        let types = parts.iter().any(|m| !m.face_types.is_empty());
        let alphas = parts.iter().any(|m| !m.alphas.is_empty());
        let mut out =
            Mesh { vertices: Vec::new(), faces: Vec::new(), colours: Vec::new(), face_types: Vec::new(), alphas: Vec::new(), version: 12 };
        for m in parts {
            for (i, face) in m.faces.iter().enumerate() {
                if types {
                    out.face_types.push(m.face_types.get(i).copied().unwrap_or(0));
                }
                if alphas {
                    out.alphas.push(m.alphas.get(i).copied().unwrap_or(0));
                }
                let corner = |out: &mut Mesh, v: u16| {
                    let p = m.vertices[v as usize];
                    match out.vertices.iter().position(|&q| q == p) {
                        Some(k) => k as u16,
                        None => {
                            out.vertices.push(p);
                            (out.vertices.len() - 1) as u16
                        }
                    }
                };
                let f = [corner(&mut out, face[0]), corner(&mut out, face[1]), corner(&mut out, face[2])];
                out.faces.push(f);
                out.colours.push(m.colours[i]);
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
        let (len_x, len_y) = (h.u16()? as usize, h.u16()? as usize);
        h.u16()?; // z length: the last section read here
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
        if textures {
            pos += face_count * 2;
        }
        let colours_at = pos + texture_indices;
        let x_at = colours_at + face_count * 2;
        let (y_at, z_at) = (x_at + len_x, x_at + len_x + len_y);
        Ok(Mesh {
            vertices: vertices(data, vertex_count, flags_at, [x_at, y_at, z_at])?,
            faces: faces(data, face_count, indices_at, strips_at)?,
            colours: colours(data, face_count, colours_at)?,
            face_types: section(data, face_types, flags_at + vertex_count, face_count)?.into_iter().map(|b| b as i8).collect(),
            alphas: section(data, alpha, alphas_at, face_count)?,
            version,
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
        pos += face_count * 2 + textured * 6;
        let (x_at, y_at) = (pos, pos + len_x);
        let z_at = y_at + len_y;
        let vertices = vertices(data, vertex_count, 0, [x_at, y_at, z_at])?;
        // Render type bit 1 makes a face flat (type 1, else 0); with bit 2 it is textured: its
        // colour field held the texture and the colour becomes 127.
        let mut colours = colours(data, face_count, colours_at)?;
        let types_at = vertex_count + face_count + usize::from(priority == 255) * face_count + usize::from(face_skins == 1) * face_count;
        let render = section(data, render_types == 1, types_at, face_count)?;
        for (c, t) in colours.iter_mut().zip(&render) {
            if t & 2 == 2 {
                *c = 127;
            }
        }
        Ok(Mesh {
            vertices,
            faces: faces(data, face_count, indices_at, strips_at)?,
            colours,
            face_types: render.iter().map(|t| (t & 1) as i8).collect(),
            alphas: section(data, alpha == 1, alphas_at, face_count)?,
            version: 12,
        })
    }
}
