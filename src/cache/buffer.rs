//! Reading big-endian numbers out of a byte slice.

use super::error::{corrupt, Result};

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.remaining() < n {
            return corrupt(format!("wanted {n} bytes at offset {}, only {} left", self.pos, self.remaining()));
        }
        let out = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    fn uint(&mut self, width: usize) -> Result<u32> {
        Ok(self.bytes(width)?.iter().fold(0u32, |acc, &b| (acc << 8) | b as u32))
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16> {
        Ok(self.uint(2)? as u16)
    }

    pub fn u32(&mut self) -> Result<u32> {
        self.uint(4)
    }

    pub fn i32(&mut self) -> Result<i32> {
        Ok(self.u32()? as i32)
    }

    /// "Big smart": 2 bytes if the top bit is clear, otherwise 4 bytes with the top bit masked off.
    pub fn big_smart(&mut self) -> Result<u32> {
        if self.data.get(self.pos).is_some_and(|b| b & 0x80 != 0) {
            Ok(self.u32()? & 0x7fff_ffff)
        } else {
            Ok(self.u16()? as u32)
        }
    }

    /// The client's `readBigSmart`: like `big_smart`, but the 2-byte value 32767 means "none" (-1).
    pub fn big_smart_or_none(&mut self) -> Result<i32> {
        if self.data.get(self.pos).is_some_and(|b| b & 0x80 != 0) {
            Ok((self.u32()? & 0x7fff_ffff) as i32)
        } else {
            let v = self.u16()?;
            Ok(if v == 32767 { -1 } else { v as i32 })
        }
    }

    pub fn i8(&mut self) -> Result<i8> {
        Ok(self.u8()? as i8)
    }

    pub fn i16(&mut self) -> Result<i16> {
        Ok(self.u16()? as i16)
    }

    pub fn u24(&mut self) -> Result<u32> {
        self.uint(3)
    }

    pub fn i24(&mut self) -> Result<i32> {
        let v = self.u24()? as i32;
        Ok(if v > 0x7f_ffff { v - 0x100_0000 } else { v })
    }

    /// A 0-terminated string, one byte per character (Windows-1252).
    pub fn string(&mut self) -> Result<String> {
        let rest = &self.data[self.pos..];
        let Some(end) = rest.iter().position(|&b| b == 0) else { return corrupt("string has no end marker") };
        self.pos += end + 1;
        Ok(rest[..end].iter().map(|&b| crate::cache::text::cp1252_to_char(b)).collect())
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn seek(&mut self, pos: usize) {
        self.pos = pos.min(self.data.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_big_endian_and_smarts() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x00, 0x10, 0x80, 0x00, 0x01, 0x00];
        let mut r = Reader::new(&data);
        assert_eq!(r.u16().unwrap(), 0x0102);
        assert_eq!(r.i32().unwrap(), 0x03040506);
        assert_eq!(r.big_smart().unwrap(), 0x10);
        assert_eq!(r.big_smart().unwrap(), 0x100);
        assert!(r.u8().is_err());
    }
}
