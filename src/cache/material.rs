//! Textures: the texture table (index 26, `ImageIndexLoader`'s constructor) and each texture's
//! procedural material (index 9, `MaterialDefinitions`): a graph of operations
//! (`MaterialProperty.decode`), each reading its own settings, wired to the operations it takes
//! its input from, with three outputs (opaque, translucent, combined).
//!
//! Every operation's settings are read so the file stays in step (`MaterialProp0` ... `39`);
//! what drawing needs is kept as the operations get ported.

use super::buffer::Reader;
use super::error::{corrupt, Result};

/// One texture's entry in the texture table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureDetails {
    /// `isGroundMesh` (byte 0): drawn on the ground as a plain colour.
    pub ground_mesh: bool,
    /// `skipTriangles`: faces with it aren't drawn.
    pub skip_triangles: bool,
    /// `brightness`, `shadowFactor` (signed bytes).
    pub brightness: i8,
    pub shadow: i8,
    /// `color`: the texture's average colour, packed HSL.
    pub colour: u16,
    /// `isBrickTile`.
    pub brick: bool,
    /// `blendType`.
    pub blend: u8,
}

/// `ImageIndexLoader`'s constructor: a u16 count, then each field for every texture that
/// exists, field by field.
pub fn texture_table(data: &[u8]) -> Result<Vec<Option<TextureDetails>>> {
    let mut r = Reader::new(data);
    let count = r.u16()? as usize;
    let mut t: Vec<Option<TextureDetails>> = (0..count).map(|_| Ok((r.u8()? == 1).then(TextureDetails::default))).collect::<Result<_>>()?;
    let t_ = &mut t;
    each(&mut r, t_, |d, b| d.ground_mesh = b == 0)?;
    each(&mut r, t_, |_, _| {})?; // isHalfSize
    each(&mut r, t_, |d, b| d.skip_triangles = b == 1)?;
    each(&mut r, t_, |d, b| d.brightness = b as i8)?;
    each(&mut r, t_, |d, b| d.shadow = b as i8)?;
    each(&mut r, t_, |_, _| {})?; // effectId
    each(&mut r, t_, |_, _| {})?; // effectParam1
    for d in t_.iter_mut().flatten() {
        d.colour = r.u16()?;
    }
    each(&mut r, t_, |_, _| {})?; // textureSpeedU
    each(&mut r, t_, |_, _| {})?; // textureSpeedV
    each(&mut r, t_, |_, _| {})?; // aBool2087
    each(&mut r, t_, |d, b| d.brick = b == 1)?;
    each(&mut r, t_, |_, _| {})?; // useMipmaps
    each(&mut r, t_, |_, _| {})?; // repeatS
    each(&mut r, t_, |_, _| {})?; // repeatT
    each(&mut r, t_, |_, _| {})?; // hdr
    each(&mut r, t_, |_, _| {})?; // combineMode
    for _ in t_.iter().flatten() {
        r.i32()?; // effectParam2
    }
    each(&mut r, t_, |d, b| d.blend = b)?;
    Ok(t)
}

/// Reads one byte for every texture that exists into `set`.
fn each(r: &mut Reader, t: &mut [Option<TextureDetails>], mut set: impl FnMut(&mut TextureDetails, u8)) -> Result<()> {
    for d in t.iter_mut().flatten() {
        set(d, r.u8()?);
    }
    Ok(())
}

/// One operation of a material.
#[derive(Clone, Debug, PartialEq)]
pub struct Op {
    /// `MaterialProperty.getById`: 0..=39.
    pub kind: u8,
    /// The operations it takes its inputs from.
    pub inputs: Vec<usize>,
    /// A sprite (39, 18) or texture (36) it reads, −1 none.
    pub image: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub ops: Vec<Op>,
    /// `opaqueProperty`, `translucentProperty`, `combinedProperty`.
    pub outputs: [usize; 3],
}

/// How many inputs each operation takes (`MaterialProperty(numParams, ...)`).
const INPUTS: [usize; 40] =
    [0, 0, 0, 0, 0, 1, 1, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 3, 1, 3, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0];

impl Material {
    /// `MaterialDefinitions(ByteBuf)`.
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let count = r.u8()? as usize;
        let mut ops = Vec::with_capacity(count);
        for _ in 0..count {
            r.u8()?;
            let kind = r.u8()?;
            if kind >= 40 {
                return corrupt(format!("material operation {kind}"));
            }
            r.u8()?; // anInt7668: the palette size
            let mut image = -1;
            // `MaterialProp34.anInt9736`: how many curve points its setting 2 may list.
            let mut points = 4;
            let settings = r.u8()?;
            for _ in 0..settings {
                let setting = r.u8()?;
                if let Some(id) = skip_setting(&mut r, kind, setting, &mut points)? {
                    image = id;
                }
            }
            let inputs = (0..INPUTS[kind as usize]).map(|_| Ok(r.u8()? as usize)).collect::<Result<Vec<_>>>()?;
            ops.push(Op { kind, inputs, image });
        }
        let outputs = [r.u8()? as usize, r.u8()? as usize, r.u8()? as usize];
        if ops.iter().flat_map(|o| &o.inputs).chain(&outputs).any(|&i| i >= ops.len()) {
            return corrupt("material input out of range".to_string());
        }
        Ok(Material { ops, outputs })
    }
}

/// Reads setting `setting` of an operation of `kind` (each `MaterialPropN.decode`); a sprite or
/// texture id comes back. `points` is operation 34's curve length (its setting 1).
fn skip_setting(r: &mut Reader, kind: u8, setting: u8, points: &mut usize) -> Result<Option<i32>> {
    let (bytes, shorts): (usize, usize) = match (kind, setting) {
        (0, 0) | (5, 0..=2) | (6, 2) | (7, 0..=1) | (9, 0..=2) | (12, 0) | (12, 1) | (12, 3) | (15, 0) | (15, 1) | (15, 3..=6) => (1, 0),
        (16, 0..=1) | (19, 1) | (20, 0..=1) | (21, 0) | (22, 0) | (23, 0) | (27, 0) | (27, 2) | (28, 0) | (28, 6) | (29, 1) | (30, 2) => {
            (1, 0)
        }
        (33, 1) | (34, 0) | (34, 3..=6) | (38, 0) | (38, 2) => (1, 0),
        (34, 1) => {
            *points = r.u8()? as usize;
            return Ok(None);
        }
        (1, 0) => (3, 0),
        (4, 0..=1) => (1, 0),
        (4, 2..=7) | (6, 0..=1) | (11, 0..=2) | (14, 0) | (15, 2) | (16, 2) | (17, 0) | (19, 0) | (25, 0..=3) | (26, 0..=1) => (0, 1),
        (27, 1) | (28, 1..=5) | (28, 7..=8) | (30, 0..=1) | (31, 0..=3) | (32, 0..=2) | (33, 0) | (35, 0) | (37, 0..=6) => (0, 1),
        (38, 1) | (38, 3..=4) => (0, 1),
        (17, 1..=2) => (1, 0),
        (25, 4) => (3, 0),
        (8, 0) => {
            r.u8()?;
            let n = r.u8()? as usize;
            r.bytes(n * 4)?;
            return Ok(None);
        }
        (10, 0) => {
            if r.u8()? == 0 {
                let n = r.u8()? as usize;
                r.bytes(n * 5)?;
            }
            return Ok(None);
        }
        (29, 0) => {
            let n = r.u8()?;
            for _ in 0..n {
                let size = match r.u8()? {
                    0 => 12,
                    1 => 20,
                    2 | 3 => 15,
                    _ => 0,
                };
                r.bytes(size)?;
            }
            return Ok(None);
        }
        (34, 2) => {
            // `points` shorts follow when the short is negative.
            if r.i16()? < 0 {
                r.bytes(*points * 2)?;
            }
            return Ok(None);
        }
        (36, 0) | (39, 0) | (18, 0) => return Ok(Some(r.u16()? as i32)),
        // Any other setting reads nothing.
        _ => (0, 0),
    };
    r.bytes(bytes + shorts * 2)?;
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materials_wire_their_operations() {
        // Two operations: 0 (a constant, setting 0 = 128) and 5 (setting 2 = 1, input 0);
        // outputs 1, 1, 0.
        let data = [2, 0, 0, 255, 1, 0, 128, 0, 5, 255, 1, 2, 1, 0, 1, 1, 0];
        let m = Material::decode(&data).unwrap();
        assert_eq!(m.ops.iter().map(|o| (o.kind, o.inputs.clone())).collect::<Vec<_>>(), vec![(0, vec![]), (5, vec![0])]);
        assert_eq!(m.outputs, [1, 1, 0]);
        // A sprite operation keeps its sprite id.
        let sprite = Material::decode(&[1, 0, 39, 255, 1, 0, 0, 7, 0, 0, 0]).unwrap();
        assert_eq!(sprite.ops[0].image, 7);
    }

    #[test]
    fn the_texture_table_reads_field_by_field() {
        // Two slots, the second defined: ground mesh (0 → true), half size, skip (1), brightness
        // 5, shadow −1, effect, effect param, colour 0x1234, speeds, aBool, brick (1), mipmaps,
        // repeat S/T, hdr, combine, effectParam2 (int), blend 2.
        let mut data = vec![0, 2, 0, 1, 0, 0, 1, 5, 0xff, 0, 0, 0x12, 0x34, 0, 0, 0, 1, 0, 0, 0, 0, 0];
        data.extend([0, 0, 0, 0, 2]);
        let t = texture_table(&data).unwrap();
        assert_eq!(t[0], None);
        let d = t[1].clone().unwrap();
        assert_eq!(
            (d.ground_mesh, d.skip_triangles, d.brightness, d.shadow, d.colour, d.brick, d.blend),
            (true, true, 5, -1, 0x1234, true, 2)
        );
    }
}
