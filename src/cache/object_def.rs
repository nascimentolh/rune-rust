//! Object (location) definitions: index 16, archive id >> 8, file id & 0xff
//! (`SharedConfigsType.OBJECTS`). Port of `ObjectDefinition.method7965` and `method7966`: every
//! opcode is read so the file stays in step; what the client uses so far is kept.

use super::buffer::Reader;
use super::error::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectDef {
    /// `name` (opcode 2), "null" when unnamed.
    pub name: String,
    /// `types` / `modelIds` (opcode 1): the models for each placement type (−1 is "none").
    pub models: Vec<(i8, Vec<i32>)>,
    /// `sizeX`, `sizeY` (opcodes 14, 15), in tiles.
    pub size: (i32, i32),
    /// `interactable` (opcode 19, else `method7966`: 1 for a lone scenery model or any option).
    pub interactable: i32,
    /// `clipType` (2; opcode 17 makes it 0, 27 makes it 1; `ignoreAltClip`, 74, makes it 0).
    pub clip_type: i32,
    /// `obstructsGround` (opcode 73).
    pub obstructs_ground: bool,
    /// `ambient` (opcode 29, a signed byte) and `contrast` (opcode 39, a signed byte × 5).
    pub ambient: i32,
    pub contrast: i32,
    /// `originalColors` → `modifiedColors` (opcode 40).
    pub recolours: Vec<(u16, u16)>,
    /// `aByteArray5641` (opcode 42): recolour `k` takes `aShortArray5691[byte]` instead (an
    /// array the client never fills, so 0).
    pub recolour_palette: Vec<u8>,
    /// `inverted` (opcode 62): the model is mirrored.
    pub inverted: bool,
    /// `scaleX/Y/Z` (opcodes 65..=67, 128 is 1).
    pub scale: [i32; 3],
    /// `offsetX/Y/Z` (opcodes 70..=72, signed × 4).
    pub offset: [i32; 3],
    /// `groundContoured` (21 → 1, 81 → 2, 93 / 162 → 3, 94 → 4, 95 → 5) and its parameter
    /// `anInt5654` (−1 unless given).
    pub contoured: u8,
    pub contour_param: i32,
    /// `anInt5682`, `anInt5683`, `anInt5710` (opcodes 164..=166): moved after contouring.
    pub nudge: [i32; 3],
    /// `hidden` (opcode 82): left out when textures are off.
    pub hidden: bool,
}

impl Default for ObjectDef {
    fn default() -> Self {
        ObjectDef {
            name: "null".into(),
            models: Vec::new(),
            size: (1, 1),
            interactable: -1,
            clip_type: 2,
            obstructs_ground: false,
            ambient: 0,
            contrast: 0,
            recolours: Vec::new(),
            recolour_palette: Vec::new(),
            inverted: false,
            scale: [128; 3],
            offset: [0; 3],
            contoured: 0,
            contour_param: -1,
            nudge: [0; 3],
            hidden: false,
        }
    }
}

impl ObjectDef {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut d = ObjectDef::default();
        let (mut has_option, mut ignore_alt_clip) = (false, false);
        loop {
            let op = r.u8()?;
            match op {
                0 => break,
                1 => {
                    let count = r.u8()?;
                    d.models = (0..count)
                        .map(|_| {
                            let kind = r.i8()?;
                            let n = r.u8()?;
                            let ids = (0..n).map(|_| r.big_smart_or_none()).collect::<Result<Vec<_>>>()?;
                            Ok((kind, ids))
                        })
                        .collect::<Result<Vec<_>>>()?;
                }
                2 => d.name = r.string()?,
                14 => d.size.0 = r.u8()? as i32,
                15 => d.size.1 = r.u8()? as i32,
                17 => d.clip_type = 0,
                19 => d.interactable = r.u8()? as i32,
                21 => d.contoured = 1,
                27 => d.clip_type = 1,
                29 => d.ambient = r.i8()? as i32,
                39 => d.contrast = r.i8()? as i32 * 5,
                30..=34 | 150..=154 => {
                    r.string()?;
                    has_option = true;
                }
                40 => {
                    let n = r.u8()?;
                    d.recolours = (0..n).map(|_| Ok((r.u16()?, r.u16()?))).collect::<Result<Vec<_>>>()?;
                }
                42 => {
                    let n = r.u8()?;
                    d.recolour_palette = r.bytes(n as usize)?.to_vec();
                }
                62 => d.inverted = true,
                65..=67 => d.scale[op as usize - 65] = r.u16()? as i32,
                70..=72 => d.offset[op as usize - 70] = (r.i16()? as i32) << 2,
                73 => d.obstructs_ground = true,
                74 => ignore_alt_clip = true,
                81 => {
                    d.contoured = 2;
                    d.contour_param = r.u8()? as i32 * 256;
                }
                82 => d.hidden = true,
                93 => {
                    d.contoured = 3;
                    d.contour_param = r.u16()? as i32;
                }
                94 => d.contoured = 4,
                95 => {
                    d.contoured = 5;
                    d.contour_param = r.i16()? as i32;
                }
                162 => {
                    d.contoured = 3;
                    d.contour_param = r.i32()?;
                }
                164..=166 => d.nudge[op as usize - 164] = r.i16()? as i32,
                // Flags kept for later: blocking, shading, occluding, shadows, sounds, map icons...
                18 | 22 | 23 | 64 | 88 | 89 | 91 | 97 | 98 | 103 | 105 | 168 | 169 | 177 | 189 => {}
                // One byte: decoration offset, map sprite rotation, sound volume, ...
                28 | 69 | 75 | 101 | 104 | 178 => {
                    r.u8()?;
                }
                // Two bytes: map sprite, map icon, ...
                102 | 107 | 167 => {
                    r.u16()?;
                }
                24 => {
                    r.big_smart_or_none()?;
                }
                41 => {
                    let n = r.u8()?;
                    r.bytes(n as usize * 4)?;
                }
                78 => {
                    r.u16()?;
                    r.u8()?;
                }
                79 => {
                    r.u16()?;
                    r.u16()?;
                    r.u8()?;
                    let n = r.u8()?;
                    r.bytes(n as usize * 2)?;
                }
                77 | 92 => {
                    r.u16()?;
                    r.u16()?;
                    if op == 92 {
                        r.big_smart_or_none()?;
                    }
                    let n = r.u8()?;
                    for _ in 0..=n {
                        r.big_smart_or_none()?;
                    }
                }
                99 | 100 => {
                    r.u8()?;
                    r.u16()?;
                }
                106 => {
                    let n = r.u8()?;
                    for _ in 0..n {
                        r.big_smart_or_none()?;
                        r.u8()?;
                    }
                }
                160 => {
                    let n = r.u8()?;
                    r.bytes(n as usize * 2)?;
                }
                163 | 173 => {
                    r.bytes(4)?;
                }
                170 | 171 => {
                    r.smart()?;
                }
                249 => {
                    let n = r.u8()?;
                    for _ in 0..n {
                        let text = r.u8()? == 1;
                        r.u24()?;
                        if text {
                            r.string()?;
                        } else {
                            r.i32()?;
                        }
                    }
                }
                // `method7965` skips any other opcode without reading.
                _ => {}
            }
        }
        // `method7966`, then `getObjectDefs`' `ignoreAltClip`.
        if d.interactable == -1 {
            d.interactable = i32::from((d.models.len() == 1 && d.models[0].0 == 10) || has_option);
        }
        if ignore_alt_clip {
            d.clip_type = 0;
        }
        Ok(d)
    }

    /// `ObjectDefinition.method7971`'s pick: the model ids for placement type `kind`.
    pub fn models_for(&self, kind: i8) -> Option<&[i32]> {
        self.models.iter().find(|(k, _)| *k == kind).map(|(_, ids)| ids.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objects_read_their_models_and_name() {
        // Opcode 1: one type (10) with models 5 and none (32767); name "Tree"; size 2; an
        // option; then the end.
        let data = [1, 1, 10, 2, 0, 5, 0x7f, 0xff, 2, b'T', b'r', b'e', b'e', 0, 14, 2, 30, b'C', 0, 0];
        let d = ObjectDef::decode(&data).unwrap();
        assert_eq!((d.name.as_str(), d.size, d.interactable), ("Tree", (2, 1), 1));
        assert_eq!(d.models_for(10), Some(&[5, -1][..]));
        assert_eq!(d.models_for(22), None);
        assert_eq!(ObjectDef::decode(&[0]).unwrap().name, "null");
    }

    #[test]
    fn objects_read_their_look() {
        // Ambient −2, contrast 3 (× 5), offset y −1 (× 4), contoured with 81 (× 256), mirrored.
        let data = [29, 0xfe, 39, 3, 71, 0xff, 0xff, 81, 2, 62, 0];
        let d = ObjectDef::decode(&data).unwrap();
        assert_eq!((d.ambient, d.contrast, d.offset, d.contoured, d.contour_param, d.inverted), (-2, 15, [0, -4, 0], 2, 512, true));
    }
}
