//! The entity defaults file (index 28, file 3). Port of `EntityDefaults.decode`; the login
//! window and the profiling model are kept, the other opcodes are read past.

use super::buffer::Reader;
use super::error::{corrupt, Result};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Defaults {
    /// `loginWindow` (opcode 5).
    pub login_window: Option<u32>,
    /// `profilingModel` (opcode 2): the mesh Auto Setup draws to time a graphics toolkit.
    pub profiling_model: Option<u32>,
}

pub fn decode(data: &[u8]) -> Result<Defaults> {
    let mut r = Reader::new(data);
    let mut hits = 4;
    let mut d = Defaults::default();
    loop {
        match r.u8()? {
            0 => return Ok(d),
            1 => {
                r.bytes(hits * 4)?;
            }
            2 => d.profiling_model = u32::try_from(r.big_smart_or_none()?).ok(),
            3 => hits = r.u8()? as usize,
            4 | 8 | 10 => {}
            5 => d.login_window = Some(r.u24()?),
            6 => {
                r.u24()?;
            }
            7 => {
                for _ in 0..40 {
                    r.u16()?;
                    let n = r.u16()? as usize;
                    r.bytes(n * 2)?;
                }
            }
            9 | 11 | 13 | 14 | 15 => {
                r.u8()?;
            }
            12 => {
                r.bytes(4)?;
            }
            other => return corrupt(format!("unknown entity defaults opcode {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_login_window_after_other_opcodes() {
        let data = [3, 2, 1, 0, 1, 0, 2, 0, 3, 0, 4, 9, 7, 5, 0, 2, 0xe8, 0];
        assert_eq!(decode(&data).unwrap(), Defaults { login_window: Some(744), profiling_model: None });
        assert_eq!(decode(&[2, 0x80, 0, 0x30, 0x39, 0]).unwrap().profiling_model, Some(12345));
    }
}
