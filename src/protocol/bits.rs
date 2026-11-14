//! Bit access in a packet (`ByteBuf.Bit`): `REGION` and `PLAYER_UPDATE` write and read fields
//! of any width, most significant bit first, between `initBitAccess` and `finishBitAccess`
//! (which rounds up to the next whole byte).

/// `writeBits`: fields packed high bit first; the last byte is padded with 0 bits.
#[derive(Default)]
pub struct BitWriter {
    pub bytes: Vec<u8>,
    at: usize,
}

impl BitWriter {
    pub fn write(&mut self, count: u32, value: u32) {
        for i in (0..count).rev() {
            if self.at.is_multiple_of(8) {
                self.bytes.push(0);
            }
            if value >> i & 1 == 1 {
                let last = self.bytes.len() - 1;
                self.bytes[last] |= 0x80 >> (self.at % 8);
            }
            self.at += 1;
        }
    }
}

/// `readBits` over `data` from byte `start`; `finish` is the byte after the last one used
/// (`finishBitAccess`).
pub struct BitReader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8], start: usize) -> Self {
        BitReader { data, at: start * 8 }
    }

    /// `None` when the data runs out.
    pub fn read(&mut self, count: u32) -> Option<u32> {
        let mut value = 0;
        for _ in 0..count {
            let byte = *self.data.get(self.at / 8)?;
            value = value << 1 | u32::from(byte >> (7 - self.at % 8) & 1);
            self.at += 1;
        }
        Some(value)
    }

    pub fn finish(&self) -> usize {
        self.at.div_ceil(8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_go_high_bit_first_and_read_back() {
        let mut b = BitWriter::default();
        b.write(3, 0b101);
        b.write(6, 0b111111);
        assert_eq!(b.bytes, vec![0b1011_1111, 0b1000_0000]);
        let mut r = BitReader::new(&b.bytes, 0);
        assert_eq!((r.read(3), r.read(6), r.finish()), (Some(0b101), Some(0b111111), 2));
        assert_eq!(r.read(8), None);
    }
}
