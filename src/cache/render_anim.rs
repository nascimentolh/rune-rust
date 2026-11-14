//! Base animation sets (`BASDefinitions`, config archive 32: `SharedConfigsType.RENDER_ANIMS`,
//! one file per id). Port of `method3821` / `method3808`: every opcode is read, and the values
//! the movement uses are kept (how fast the entity turns, and how it speeds up along its path).
//! The animations, model offsets and the rest are read past for now. An opcode it doesn't know
//! reads nothing, as the reference's `if` chain does.

use super::buffer::Reader;
use super::error::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderAnim {
    /// `anInt2820` (opcode 29): how fast the facing speeds up while turning; 0 = the entity's
    /// own turn speed (`anInt10340`).
    pub turn_acceleration: i32,
    /// `anInt2804` (opcode 30): the most the facing turns by in a cycle (0 = no limit).
    pub turn_speed: i32,
    /// `anInt2826` (opcode 37): how fast the step speed eases up and down; −1 = it doesn't ease.
    pub move_acceleration: i32,
}

impl Default for RenderAnim {
    /// A new `BASDefinitions` (also `RenderAnimIndexLoader.aClass227_2669`, used with no set).
    fn default() -> Self {
        RenderAnim { turn_acceleration: 0, turn_speed: 0, move_acceleration: -1 }
    }
}

/// `readBigSmart`: a u16, or a u32 with the top bit when the first byte has it.
fn big_smart(r: &mut Reader) -> Result<()> {
    if r.peek_u8()? & 0x80 != 0 {
        r.u32()?;
    } else {
        r.u16()?;
    }
    Ok(())
}

pub fn decode(data: &[u8]) -> Result<RenderAnim> {
    let mut r = Reader::new(data);
    let mut d = RenderAnim::default();
    loop {
        match r.u8()? {
            0 => return Ok(d),
            1 | 2 => {
                big_smart(&mut r)?;
                big_smart(&mut r)?;
            }
            3..=9 | 38..=42 | 46..=51 => big_smart(&mut r)?,
            26 | 54 => {
                r.bytes(2)?;
            }
            27 => {
                r.u8()?;
                r.bytes(12)?;
            }
            28 => {
                let n = usize::from(r.u8()?);
                r.bytes(n)?;
            }
            29 => d.turn_acceleration = i32::from(r.u8()?),
            30 => d.turn_speed = i32::from(r.u16()?),
            31 | 34 => {
                r.u8()?;
            }
            32 | 33 | 35 | 36 | 43 | 44 | 45 => {
                r.u16()?;
            }
            37 => d.move_acceleration = i32::from(r.u8()?),
            52 => {
                for _ in 0..r.u8()? {
                    big_smart(&mut r)?;
                    r.u8()?;
                }
            }
            55 => {
                r.u8()?;
                r.u16()?;
            }
            56 => {
                r.u8()?;
                r.bytes(6)?;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_movement_values_are_found_among_the_others() {
        // 1: stand 808, walk as a 4-byte big smart; 26: two bytes; 29: 64; 30: 512; 37: 16;
        // 52: one random stand; 53: no data.
        let data = [1, 0x03, 0x28, 0x80, 0, 0x03, 0x33, 26, 1, 2, 29, 64, 30, 2, 0, 37, 16, 52, 1, 0, 5, 9, 53, 0];
        assert_eq!(decode(&data).unwrap(), RenderAnim { turn_acceleration: 64, turn_speed: 512, move_acceleration: 16 });
        assert_eq!(decode(&[0]).unwrap(), RenderAnim::default());
    }
}
