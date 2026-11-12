//! Font metrics (index 13): glyph widths, optional kerning, line spacing.
//! Port of `FontMetrics` and `Utils.calculateKerning`.
//!
//! Layout: 0 · variable-width flag · 256 glyph widths · [if variable: 256 left-edge counts,
//! 256 left-edge starts, then delta-coded left and right edge profiles] · [else: spacing] ·
//! u8 · u8 · top padding · bottom padding.

use super::buffer::Reader;
use super::error::{corrupt, Result};
use super::text::char_to_cp1252;

pub struct FontMetrics {
    pub glyph_widths: [u8; 256],
    /// kerning[previous][next], only for variable-width fonts.
    kerning: Option<Box<[[i8; 256]; 256]>>,
    pub vertical_spacing: i32,
    pub top_padding: i32,
    pub bottom_padding: i32,
}

const SPACE: usize = 32;
const NBSP: usize = 160;

impl FontMetrics {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        if r.u8()? != 0 {
            return corrupt("font metrics don't start with 0");
        }
        let variable = r.u8()? == 1;
        let mut glyph_widths = [0u8; 256];
        glyph_widths.copy_from_slice(r.bytes(256)?);
        let (kerning, vertical_spacing) = if variable {
            let counts: Vec<usize> = r.bytes(256)?.iter().map(|&b| b as usize).collect();
            let starts: Vec<i32> = r.bytes(256)?.iter().map(|&b| b as i32).collect();
            let profile = |r: &mut Reader| -> Result<Vec<Vec<i8>>> {
                (0..256)
                    .map(|g| {
                        let mut acc = 0i8;
                        (0..counts[g])
                            .map(|_| {
                                acc = acc.wrapping_add(r.i8()?);
                                Ok(acc)
                            })
                            .collect()
                    })
                    .collect()
            };
            let left = profile(&mut r)?;
            let right = profile(&mut r)?;
            let mut table = Box::new([[0i8; 256]; 256]);
            for a in (0..256).filter(|&g| g != SPACE && g != NBSP) {
                for b in (0..256).filter(|&g| g != SPACE && g != NBSP) {
                    table[a][b] = kerning_between(&left, &right, &starts, &glyph_widths, &counts, a, b) as i8;
                }
            }
            (Some(table), starts[SPACE] + counts[SPACE] as i32)
        } else {
            (None, r.u8()? as i32)
        };
        r.u8()?;
        r.u8()?;
        let top_padding = r.u8()? as i32;
        let bottom_padding = r.u8()? as i32;
        Ok(FontMetrics { glyph_widths, kerning, vertical_spacing, top_padding, bottom_padding })
    }

    pub fn advance(&self, glyph: u8) -> i32 {
        self.glyph_widths[glyph as usize] as i32
    }

    pub fn kerning(&self, previous: u8, next: u8) -> i32 {
        self.kerning.as_ref().map_or(0, |k| k[previous as usize][next as usize] as i32)
    }

    /// Width of a line, skipping `<tags>` and turning entity tags into their characters.
    pub fn text_width(&self, text: &str) -> i32 {
        let mut width = 0;
        let mut previous: Option<u8> = None;
        for token in tokens(text) {
            if let Token::Glyph(g) = token {
                width += self.advance(g);
                if let Some(p) = previous {
                    width += self.kerning(p, g);
                }
                previous = Some(g);
            }
        }
        width
    }

    /// Breaks text into lines no wider than `max_width` at spaces or after hyphens;
    /// `<br>` forces a break. Port of `splitLines(..., true)`, which drops the space at a break.
    pub fn split_lines(&self, text: &str, max_width: Option<i32>) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut lines = Vec::new();
        let (mut line_start, mut width) = (0usize, 0i32);
        let mut break_at: Option<usize> = None;
        let (mut width_at_break, mut drop_space) = (0i32, 0usize);
        let mut previous: Option<u8> = None;
        let mut tag_start: Option<usize> = None;
        for i in 0..chars.len() {
            let byte = char_to_cp1252(chars[i]);
            if byte == b'<' {
                tag_start = Some(i);
                continue;
            }
            let (glyph, char_start) = match tag_start {
                Some(start) => {
                    if byte != b'>' {
                        continue;
                    }
                    tag_start = None;
                    let tag: String = chars[start + 1..i].iter().collect();
                    if tag == "br" {
                        lines.push(chars[line_start..=i].iter().collect());
                        line_start = i + 1;
                        width = 0;
                        break_at = None;
                        previous = None;
                        continue;
                    }
                    (entity(&tag), start)
                }
                None => (Some(byte), i),
            };
            let Some(g) = glyph else { continue };
            let mut step = self.advance(g);
            if let Some(p) = previous {
                step += self.kerning(p, g);
            }
            previous = Some(g);
            if step <= 0 {
                continue;
            }
            width += step;
            let Some(max) = max_width else { continue };
            // Only real characters (not entity tags) can be break points.
            let is_char = tag_start.is_none() && char_start == i;
            if is_char && g == b' ' {
                break_at = Some(i);
                width_at_break = width;
                drop_space = 1;
            }
            if width > max {
                match break_at {
                    Some(b) => {
                        lines.push(chars[line_start..b + 1 - drop_space].iter().collect());
                        line_start = b + 1;
                        width -= width_at_break;
                    }
                    None => {
                        lines.push(chars[line_start..char_start].iter().collect());
                        line_start = char_start;
                        width = step;
                    }
                }
                break_at = None;
                previous = None;
            }
            if is_char && g == b'-' {
                break_at = Some(i);
                width_at_break = width;
                drop_space = 0;
            }
        }
        if chars.len() > line_start {
            lines.push(chars[line_start..].iter().collect());
        }
        lines
    }
}

/// Port of `Utils.calculateKerning`: how far two glyphs can move together without their
/// edge profiles overlapping, over the rows both cover.
fn kerning_between(left: &[Vec<i8>], right: &[Vec<i8>], starts: &[i32], widths: &[u8; 256], counts: &[usize], a: usize, b: usize) -> i32 {
    let (a_top, b_top) = (starts[a], starts[b]);
    let (a_bottom, b_bottom) = (a_top + counts[a] as i32, b_top + counts[b] as i32);
    let top = a_top.max(b_top);
    let bottom = a_bottom.min(b_bottom);
    let mut gap = (widths[a] as i32).min(widths[b] as i32);
    for row in top..bottom {
        let sum = right[a][(row - a_top) as usize] as i32 + left[b][(row - b_top) as usize] as i32;
        gap = gap.min(sum);
    }
    -gap
}

/// A piece of text: a glyph to draw, or a formatting tag.
#[derive(Debug, PartialEq)]
pub enum Token {
    Glyph(u8),
    Tag(String),
}

/// Named character tags the client understands.
fn entity(tag: &str) -> Option<u8> {
    Some(match tag {
        "lt" => b'<',
        "gt" => b'>',
        "nbsp" => 160,
        "shy" => 173,
        "times" => 215,
        "euro" => 0x80,
        "copy" => 169,
        "reg" => 174,
        _ => return None,
    })
}

/// Splits text into glyphs and `<tags>`; entity tags become glyphs. An unclosed `<` swallows
/// the rest, as in the client.
pub fn tokens(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut tag: Option<String> = None;
    for c in text.chars() {
        match (&mut tag, c) {
            (None, '<') => tag = Some(String::new()),
            (None, _) => out.push(Token::Glyph(char_to_cp1252(c))),
            (Some(t), '>') => {
                let t = std::mem::take(t);
                tag = None;
                out.push(match entity(&t) {
                    Some(g) => Token::Glyph(g),
                    None => Token::Tag(t),
                });
            }
            (Some(t), _) => t.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(width: u8) -> FontMetrics {
        FontMetrics { glyph_widths: [width; 256], kerning: None, vertical_spacing: 12, top_padding: 2, bottom_padding: 3 }
    }

    #[test]
    fn widths_skip_tags_and_count_entities() {
        let m = fixed(5);
        assert_eq!(m.text_width("ab"), 10);
        assert_eq!(m.text_width("<col=ff0000>ab</col>"), 10);
        assert_eq!(m.text_width("a&<gt>"), 15);
    }

    #[test]
    fn splits_at_spaces_and_breaks() {
        let m = fixed(5);
        assert_eq!(m.split_lines("aaa bbb ccc", Some(40)), vec!["aaa bbb", "ccc"]);
        assert_eq!(m.split_lines("aa<br>bb", Some(100)), vec!["aa<br>", "bb"]);
        assert_eq!(m.split_lines("abcdefghij", Some(20)), vec!["abcd", "efgh", "ij"]);
    }
}
