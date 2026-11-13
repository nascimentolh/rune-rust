//! Object (location) definitions: index 16, archive id >> 8, file id & 0xff
//! (`SharedConfigsType.OBJECTS`). Port of `ObjectDefinition.method7965`: every opcode is read so
//! the file stays in step; only what the client uses so far is kept.

use super::buffer::Reader;
use super::error::Result;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectDef {
    /// `name` (opcode 2), "null" when unnamed.
    pub name: String,
    /// `types` / `modelIds` (opcode 1): the models for each placement type (−1 is "none").
    pub models: Vec<(i8, Vec<i32>)>,
}

impl ObjectDef {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut d = ObjectDef { name: "null".into(), models: Vec::new() };
        loop {
            let op = r.u8()?;
            match op {
                0 => return Ok(d),
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
                // Flags with no value: clipping, shading, occluding, inverted, shadows...
                17 | 18 | 21 | 22 | 23 | 27 | 62 | 64 | 73 | 74 | 82 | 88 | 89 | 91 | 94 | 97 | 98 | 103 | 105 | 168 | 169 | 177 | 189 => {}
                // One byte: size, interaction, decoration offset, ambient, contrast, ...
                14 | 15 | 19 | 28 | 29 | 39 | 69 | 75 | 81 | 101 | 104 | 178 => {
                    r.u8()?;
                }
                // Two bytes: scale, offsets (signed), map sprite, map icon, ...
                65..=67 | 70..=72 | 93 | 95 | 102 | 107 | 164..=167 => {
                    r.u16()?;
                }
                24 => {
                    r.big_smart_or_none()?;
                }
                30..=34 | 150..=154 => {
                    r.string()?;
                }
                40 | 41 => {
                    let n = r.u8()?;
                    r.bytes(n as usize * 4)?;
                }
                42 => {
                    let n = r.u8()?;
                    r.bytes(n as usize)?;
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
                162 | 173 | 163 => {
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
        assert_eq!(d.name, "Tree");
        assert_eq!(d.models, vec![(10, vec![5, -1])]);
        assert_eq!(ObjectDef::decode(&[0]).unwrap().name, "null");
    }
}
