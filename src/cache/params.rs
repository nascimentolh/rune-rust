//! Params and structs (index 2, archives 11 and 26, one file per id). Port of
//! `ParamDefinitions` and `CacheableNode_Sub4` (a struct: param id → int or string).

use super::buffer::Reader;
use super::enums::EnumValue;
use super::error::Result;
use super::text::cp1252_to_char;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Param {
    type_char: char,
    pub default_int: i32,
    /// `typeName`: the default for string params.
    pub default_text: Option<String>,
}

impl Param {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut p = Param::default();
        loop {
            match r.u8()? {
                0 => return Ok(p),
                1 => p.type_char = cp1252_to_char(r.u8()?),
                2 => p.default_int = r.i32()?,
                5 => p.default_text = Some(r.string()?),
                // 4 turns auto-disable off; other opcodes carry nothing.
                _ => {}
            }
        }
    }

    pub fn is_string(&self) -> bool {
        self.type_char == 's'
    }
}

/// A struct's values by param id (opcode 249). Other opcodes carry nothing.
pub fn decode_struct(data: &[u8]) -> Result<HashMap<i32, EnumValue>> {
    let mut r = Reader::new(data);
    let mut values = HashMap::new();
    loop {
        match r.u8()? {
            0 => return Ok(values),
            249 => {
                for _ in 0..r.u8()? {
                    let is_string = r.u8()? == 1;
                    let key = r.u24()? as i32;
                    let v = if is_string { EnumValue::Text(r.string()?) } else { EnumValue::Int(r.i32()?) };
                    values.insert(key, v);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_struct_values() {
        let data = [249, 2, 0, 0, 0, 7, 0, 0, 0, 9, 1, 0, 0, 8, b'h', b'i', 0, 0];
        let s = decode_struct(&data).unwrap();
        assert_eq!(s[&7], EnumValue::Int(9));
        assert_eq!(s[&8], EnumValue::Text("hi".into()));
    }
}
