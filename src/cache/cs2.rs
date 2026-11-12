//! CS2 client scripts (index 12, one archive per script). Port of `CS2Script`:
//! `decodeHeader` reads the trailer, `decodeInstruction` each instruction.
//!
//! Layout: name (`readNullString`) · instructions · trailer: instruction count i32 · int,
//! string and long locals u16 × 3 · int, string and long arguments u16 × 3 · switch tables
//! (count u8, then per table: entries u16, then (value i32, jump i32) pairs) · trailer length u16.
//! Each instruction: opcode u16 · operand: a string for PUSH_STRING, an i64 for PUSH_LONG, an
//! i32 when the opcode has an int constant, otherwise one unsigned byte.

use super::buffer::Reader;
use super::cs2_opcodes::OPCODES;
use super::error::{corrupt, Result};
use std::collections::HashMap;

pub const PUSH_STRING: u16 = 57;
pub const PUSH_LONG: u16 = 633;
pub const CALL_CS2: u16 = 902;

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Int(i32),
    Text(String),
    Long(i64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instruction {
    pub opcode: u16,
    pub operand: Operand,
}

impl Instruction {
    pub fn name(&self) -> &'static str {
        OPCODES[self.opcode as usize].0
    }

    pub fn int(&self) -> i32 {
        match self.operand {
            Operand::Int(v) => v,
            _ => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Script {
    pub name: Option<String>,
    pub int_locals: u16,
    pub string_locals: u16,
    pub long_locals: u16,
    pub int_args: u16,
    pub string_args: u16,
    pub long_args: u16,
    /// Per switch: value → jump offset.
    pub switches: Vec<HashMap<i32, i32>>,
    pub instructions: Vec<Instruction>,
}

/// The client's text tweak (`CS2Script.decodeInstruction`): the game's name becomes "Open 727".
fn rename(text: String) -> String {
    if text.to_lowercase().contains("runescape") {
        text.replace("runescape", "Open 727").replace("RuneScape", "Open 727").replace("Runescape", "Open 727")
    } else {
        text
    }
}

impl Script {
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return corrupt("script too short");
        }
        let mut r = Reader::new(data);
        r.seek(data.len() - 2);
        let trailer_len = r.u16()? as usize;
        let Some(end) = data.len().checked_sub(2 + trailer_len + 16) else { return corrupt("script trailer doesn't fit") };
        r.seek(end);
        let count = r.i32()? as usize;
        let (int_locals, string_locals, long_locals) = (r.u16()?, r.u16()?, r.u16()?);
        let (int_args, string_args, long_args) = (r.u16()?, r.u16()?, r.u16()?);
        let switch_count = r.u8()?;
        let mut switches = Vec::with_capacity(switch_count as usize);
        for _ in 0..switch_count {
            let n = r.u16()?;
            let mut table = HashMap::new();
            for _ in 0..n {
                let value = r.i32()?;
                let jump = r.i32()?;
                table.insert(value, jump);
            }
            switches.push(table);
        }

        r.seek(0);
        // `readNullString`: an empty name is stored as a lone 0 byte.
        let name = Some(r.string()?).filter(|n| !n.is_empty());
        let mut instructions = Vec::with_capacity(count);
        while r.position() < end {
            let opcode = r.u16()?;
            let Some(&(_, has_int)) = OPCODES.get(opcode as usize) else { return corrupt(format!("unknown CS2 opcode {opcode}")) };
            let operand = match opcode {
                PUSH_STRING => Operand::Text(rename(r.string()?)),
                PUSH_LONG => Operand::Long(((r.u32()? as i64) << 32) | r.u32()? as i64),
                _ if has_int => Operand::Int(r.i32()?),
                _ => Operand::Int(r.u8()? as i32),
            };
            instructions.push(Instruction { opcode, operand });
        }
        Ok(Script { name, int_locals, string_locals, long_locals, int_args, string_args, long_args, switches, instructions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_push_int_and_return() {
        // name "" · PUSH_INT(79) 7 · RETURN(184) byte 0 · trailer
        let mut d = vec![0u8];
        d.extend([0, 79, 0, 0, 0, 7, 0, 184, 0]);
        d.extend([0, 0, 0, 2]); // 2 instructions
        d.extend([0; 12]); // locals and arguments
        d.push(0); // no switches
        d.extend([0, 1]); // switch data length, counting the switch-count byte
        let s = Script::decode(&d).unwrap();
        assert_eq!(s.instructions[0], Instruction { opcode: 79, operand: Operand::Int(7) });
        assert_eq!(s.instructions[1].name(), "RETURN");
        assert!(s.name.is_none());
    }
}
