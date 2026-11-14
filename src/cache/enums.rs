//! Enums (index 17: archive id >> 8, file id & 0xff). Port of `EnumDefinitions`.

use super::buffer::Reader;
use super::error::{corrupt, Result};
use super::text::cp1252_to_char;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum EnumValue {
    Int(i32),
    Text(String),
}

#[derive(Debug, Default)]
pub struct Enum {
    pub key_type: char,
    pub value_type: char,
    default_text: Option<String>,
    default_int: i32,
    /// `size`: how many values were stored.
    pub size: i32,
    /// Opcodes 5/6: key → value.
    map: HashMap<i32, EnumValue>,
    /// Opcodes 7/8: values by key, from 0.
    list: Option<Vec<Option<EnumValue>>>,
}

impl Enum {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut e = Enum::default();
        loop {
            match r.u8()? {
                0 => return Ok(e),
                1 => e.key_type = cp1252_to_char(r.u8()?),
                2 => e.value_type = cp1252_to_char(r.u8()?),
                3 => e.default_text = Some(r.string()?),
                4 => e.default_int = r.i32()?,
                op @ (5 | 6) => {
                    let n = r.u16()?;
                    e.size = n as i32;
                    for _ in 0..n {
                        let key = r.i32()?;
                        let v = if op == 5 { EnumValue::Text(r.string()?) } else { EnumValue::Int(r.i32()?) };
                        e.map.insert(key, v);
                    }
                }
                op @ (7 | 8) => {
                    let length = r.u16()? as usize;
                    let n = r.u16()?;
                    e.size = n as i32;
                    let mut list = vec![None; length];
                    for _ in 0..n {
                        let key = r.u16()? as usize;
                        let v = if op == 7 { EnumValue::Text(r.string()?) } else { EnumValue::Int(r.i32()?) };
                        *list.get_mut(key).ok_or(super::error::CacheError::Corrupt(format!("enum key {key} past {length}")))? = Some(v);
                    }
                    e.list = Some(list);
                }
                other => return corrupt(format!("unknown enum opcode {other}")),
            }
        }
    }

    fn value(&self, key: i32) -> Option<&EnumValue> {
        match &self.list {
            Some(list) => usize::try_from(key).ok().and_then(|k| list.get(k)).and_then(Option::as_ref),
            None => self.map.get(&key),
        }
    }

    /// `getIntValue`.
    pub fn int(&self, key: i32) -> std::result::Result<i32, String> {
        match self.value(key) {
            None => Ok(self.default_int),
            Some(EnumValue::Int(v)) => Ok(*v),
            Some(EnumValue::Text(_)) => Err("enum value is a string".into()),
        }
    }

    /// Every key the enum holds a value for, smallest first. (The reference never asks an enum
    /// what is in it; this is for listing what the cache offers, like the country flags.)
    pub fn keys(&self) -> Vec<i32> {
        let mut keys: Vec<i32> = match &self.list {
            Some(list) => list.iter().enumerate().filter(|(_, v)| v.is_some()).map(|(k, _)| k as i32).collect(),
            None => self.map.keys().copied().collect(),
        };
        keys.sort_unstable();
        keys
    }

    /// `getStringValue` (the default is "null").
    pub fn text(&self, key: i32) -> std::result::Result<String, String> {
        match self.value(key) {
            None => Ok(self.default_text.clone().unwrap_or_else(|| "null".into())),
            Some(EnumValue::Text(t)) => Ok(t.clone()),
            Some(EnumValue::Int(_)) => Err("enum value is an int".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_list_enum() {
        // key 'i', value 's', list of 3 with key 1 = "a".
        let data = [1, b'i', 2, b's', 7, 0, 3, 0, 1, 0, 1, b'a', 0, 0];
        let e = Enum::decode(&data).unwrap();
        assert_eq!((e.key_type, e.value_type, e.size), ('i', 's', 1));
        assert_eq!(e.text(1).unwrap(), "a");
        assert_eq!(e.text(2).unwrap(), "null");
    }
}
