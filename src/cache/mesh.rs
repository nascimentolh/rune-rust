//! 3D models (index 7, file 0 of each archive). Port of `RSMesh(byte[])`: `decodeNewFormat`
//! (files ending in two 0xff bytes) and `decodeOldFormat`.
//!
//! Kept: vertices, triangles and face colours, what drawing a plain mesh needs. Read past (their
//! own sections, located but not decoded): face types, priorities, alphas, skins, textures,
//! texture triangles, particle emitters and billboards.

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
        let face_types = h.u8()? & 1 == 1;
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
        for (present, n) in [(priority == 255, face_count), (face_skins, face_count), (vertex_skins, vertex_count), (alpha, face_count)] {
            if present {
                pos += n;
            }
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
        // A face with render type bit 2 is textured: its colour field held the texture and the
        // colour becomes 127.
        let mut colours = colours(data, face_count, colours_at)?;
        if render_types == 1 {
            let types_at =
                vertex_count + face_count + usize::from(priority == 255) * face_count + usize::from(face_skins == 1) * face_count;
            for (i, c) in colours.iter_mut().enumerate() {
                if data.get(types_at + i).is_some_and(|t| t & 2 == 2) {
                    *c = 127;
                }
            }
        }
        Ok(Mesh { vertices, faces: faces(data, face_count, indices_at, strips_at)?, colours })
    }
}
