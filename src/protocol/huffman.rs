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

impl Huffman {
    /// `method1973`: `count` bytes read back from `data`, following each bit (most significant
    /// first) down the codes until one ends; the bytes of `data` it took. `None` when `data` runs
    /// out first.
    pub fn decode(&self, data: &[u8], count: usize) -> Option<(Vec<u8>, usize)> {
        let by_code: std::collections::HashMap<(u8, u32), u8> =
            self.lengths.iter().enumerate().filter(|&(_, &l)| l != 0).map(|(value, &l)| ((l, self.codes[value]), value as u8)).collect();
        let mut out = Vec::with_capacity(count);
        let (mut code, mut length, mut bit) = (0u32, 0u8, 0usize);
        while out.len() < count {
            let byte = *data.get(bit / 8)?;
            if byte >> (7 - bit % 8) & 1 != 0 {
                code |= 1 << (31 - u32::from(length));
            }
            length += 1;
            bit += 1;
            if let Some(&value) = by_code.get(&(length, code)) {
                out.push(value);
                (code, length) = (0, 0);
            } else if length == 32 {
                return None;
            }
        }
        Some((out, bit.div_ceil(8)))
    }

    /// `ByteBuf.writeHuffmanString`: the text's length as a smart (`method13076`: one byte under
    /// 128, else a u16 + 32768), then its Windows-1252 bytes compressed. `None` when it is 32768
    /// bytes or more, or has a byte with no code.
    pub fn write_text(&self, text: &str) -> Option<Vec<u8>> {
        let bytes: Vec<u8> = text.chars().map(crate::cache::text::char_to_cp1252).collect();
        let length = u16::try_from(bytes.len()).ok().filter(|&n| n < 32768)?;
        let mut out = if length < 128 { vec![length as u8] } else { (length + 32768).to_be_bytes().to_vec() };
        out.extend(self.encode(&bytes)?);
        Some(out)
    }

    /// `Node_Sub33.method12582`: a smart length (at most 32767), then that many bytes
    /// decompressed, as Windows-1252. `None` where the reference catches an error and shows
    /// "Cabbage".
    pub fn read_text(&self, data: &[u8]) -> Option<String> {
        let first = *data.first()?;
        let (length, start) =
            if first < 128 { (usize::from(first), 1) } else { (usize::from(u16::from_be_bytes([first, *data.get(1)?]) - 32768), 2) };
        let (bytes, _) = self.decode(data.get(start..)?, length)?;
        Some(bytes.into_iter().map(crate::cache::text::cp1252_to_char).collect())
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
        assert_eq!(h.decode(&[0b0101_1000], 4), Some((b"abca".to_vec(), 1)));
        // 130 characters: the length takes two bytes.
        let long = "a".repeat(130);
        let wire = h.write_text(&long).unwrap();
        assert_eq!(&wire[..2], &[0x80, 130]);
        assert_eq!(h.read_text(&wire), Some(long));
        assert_eq!(h.read_text(&[4, 0b0101_1000]), Some("abca".into()));
    }
}
