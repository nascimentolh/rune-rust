//! Variable bits (index 22: group = id >> 10, file = id & 1023). Port of
//! `VarBitDefinitions.method3795`: opcode 1 gives the player variable and the bit range.

use super::buffer::Reader;
use super::error::{corrupt, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VarBit {
    /// `baseVar`: the player variable holding the bits.
    pub base: i32,
    /// `startBit`, `endBit` (both included).
    pub start: i32,
    pub end: i32,
}

pub fn decode(data: &[u8]) -> Result<VarBit> {
    let mut r = Reader::new(data);
    let mut bit = VarBit { base: 0, start: 0, end: 0 };
    loop {
        match r.u8()? {
            0 => return Ok(bit),
            1 => {
                bit.base = r.u16()? as i32;
                bit.start = r.u8()? as i32;
                bit.end = r.u8()? as i32;
            }
            other => return corrupt(format!("unknown varbit opcode {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_variable_and_bit_range() {
        assert_eq!(decode(&[1, 0x0a, 0x07, 3, 9, 0]).unwrap(), VarBit { base: 2567, start: 3, end: 9 });
    }
}
