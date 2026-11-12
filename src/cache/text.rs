//! The client's one-byte character set (Windows-1252), used by strings in the
//! cache and by fonts, whose glyphs are indexed by this byte.
//! Port of `QuestIndexLoader.getByteForChar` and its inverse.

/// Characters 0x80..=0x9F of Windows-1252 (0 = unused).
const HIGH: [char; 32] = [
    '€', '\0', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\0', 'Ž', '\0', '\0', '‘', '’', '“', '”', '•', '–', '—', '˜', '™',
    'š', '›', 'œ', '\0', 'ž', 'Ÿ',
];

pub fn cp1252_to_char(b: u8) -> char {
    match b {
        0x80..=0x9f if HIGH[(b - 0x80) as usize] != '\0' => HIGH[(b - 0x80) as usize],
        _ => b as char,
    }
}

/// The byte a character is drawn with; characters outside the set become '?'.
pub fn char_to_cp1252(c: char) -> u8 {
    match c as u32 {
        0..=0x7f | 0xa0..=0xff => c as u8,
        _ => HIGH.iter().position(|&h| h == c && h != '\0').map_or(b'?', |i| 0x80 + i as u8),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for b in 0..=255u8 {
            let c = cp1252_to_char(b);
            if !(0x80..=0x9f).contains(&b) || HIGH[(b - 0x80) as usize] != '\0' {
                assert_eq!(char_to_cp1252(c), b);
            }
        }
        assert_eq!(char_to_cp1252('€'), 0x80);
        assert_eq!(char_to_cp1252('Ω'), b'?');
    }
}
