//! `Huffman`: how chat text is compressed on the wire. The cache holds one code length per
//! byte value (index 10, the file named "huffman"; `IndexLoaders` builds the codec from it at
//! start-up); the codes are canonical in the reference's own order.

/// The codes `Huffman`'s constructor builds from the code lengths (`anIntArray1260`): each one
/// sits in the top bits of a u32.
pub struct Huffman {
    codes: Vec<u32>,
    lengths: Vec<u8>,
}

impl Huffman {
    /// `Huffman(byte[])`: for each byte value with a length, its code is the next free one of
    /// that length; then the next free codes of this and the longer lengths move on.
    pub fn new(lengths: &[u8]) -> Self {
        let mut codes = vec![0u32; lengths.len()];
        let mut next = [0u32; 33];
        for (value, &length) in lengths.iter().enumerate() {
            if length == 0 {
                continue;
            }
            let length = usize::from(length);
            let bit = 1u32 << (32 - length);
            let code = next[length];
            codes[value] = code;
            let following = if code & bit != 0 {
                next[length - 1]
            } else {
                for shorter in (1..length).rev() {
                    let c = next[shorter];
                    if c != code {
                        break;
                    }
                    let b = 1u32 << (32 - shorter);
                    if c & b != 0 {
                        next[shorter] = next[shorter - 1];
                        break;
                    }
                    next[shorter] = c | b;
                }
                code | bit
            };
            next[length] = following;
            for slot in next.iter_mut().skip(length + 1) {
                if *slot == code {
                    *slot = following;
                }
            }
        }
        Huffman { codes, lengths: lengths.to_vec() }
    }

    /// `method1971`: each byte's code, most significant bit first, packed one after another; the
    /// last byte is padded with 0 bits. `None` for a byte with no code (the reference throws).
    pub fn encode(&self, text: &[u8]) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        let mut bits = 0usize;
        for &b in text {
            let length = usize::from(*self.lengths.get(usize::from(b)).filter(|&&l| l != 0)?);
            let code = self.codes[usize::from(b)];
            for k in 0..length {
                if bits.is_multiple_of(8) {
                    out.push(0);
                }
                if code >> (31 - k) & 1 != 0 {
                    *out.last_mut()? |= 0x80 >> (bits % 8);
                }
                bits += 1;
            }
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three symbols of lengths 1, 2, 2: codes 0, 10, 11 (canonical), packed MSB first.
    #[test]
    fn codes_follow_the_lengths() {
        let mut lengths = vec![0u8; 256];
        lengths[usize::from(b'a')] = 1;
        lengths[usize::from(b'b')] = 2;
        lengths[usize::from(b'c')] = 2;
        let h = Huffman::new(&lengths);
        assert_eq!(h.codes[usize::from(b'a')], 0);
        assert_eq!(h.codes[usize::from(b'b')], 0b10 << 30);
        assert_eq!(h.codes[usize::from(b'c')], 0b11 << 30);
        // a b c a = 0 10 11 0 → 0101_1000.
        assert_eq!(h.encode(b"abca"), Some(vec![0b0101_1000]));
        assert_eq!(h.encode(b"z"), None);
    }
}
