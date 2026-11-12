//! The entity defaults file (index 28, file 3). Port of `EntityDefaults.decode`; only the
//! login window is kept, the other opcodes are read past.

use super::buffer::Reader;
use super::error::{corrupt, Result};

/// `EntityDefaults.loginWindow` (opcode 5).
pub fn login_window(data: &[u8]) -> Result<Option<u32>> {
    let mut r = Reader::new(data);
    let mut hits = 4;
    let mut login_window = None;
    loop {
        match r.u8()? {
            0 => return Ok(login_window),
            1 => {
                r.bytes(hits * 4)?;
            }
            2 => {
                r.big_smart()?;
            }
            3 => hits = r.u8()? as usize,
            4 | 8 | 10 => {}
            5 => login_window = Some(r.u24()?),
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
        assert_eq!(login_window(&data).unwrap(), Some(744));
    }
}
