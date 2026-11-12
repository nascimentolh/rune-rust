//! Drawing text with a cache font (glyph sprites from index 32, metrics from index 13).
//! Port of `FontRenderer` (`method365`, `method369`, `method371`, `method373`, `method360`)
//! with `FontRenderer_Sub1`/`Sub2` glyph tinting.

use super::raster::Raster;
use crate::cache::font_metrics::{tokens, FontMetrics, Token};
use crate::cache::sprite::Sprite;

pub struct Font {
    pub metrics: FontMetrics,
    /// One glyph per Windows-1252 byte.
    glyphs: Vec<Sprite>,
}

/// Horizontal alignment of paragraph lines.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HAlign {
    Left,
    Centre,
    Right,
    Justify,
}

/// Vertical placement of a paragraph in its box.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VAlign {
    Top,
    Centre,
    Bottom,
    Spread,
}

impl HAlign {
    pub fn from_code(code: u8) -> Self {
        [HAlign::Left, HAlign::Centre, HAlign::Right].get(code as usize).copied().unwrap_or(HAlign::Justify)
    }
}

impl VAlign {
    pub fn from_code(code: u8) -> Self {
        [VAlign::Top, VAlign::Centre, VAlign::Bottom].get(code as usize).copied().unwrap_or(VAlign::Spread)
    }
}

/// Colours that `<tags>` change while a line is drawn (`method365` / `method369`).
struct Style {
    base_colour: u32,
    colour: u32,
    base_shadow: u32,
    shadow: u32,
    strike: Option<u32>,
    underline: Option<u32>,
    /// Extra space per gap in 1/256 px, for justified lines.
    justify: i32,
    justify_acc: i32,
}

impl Style {
    fn new(colour: u32, shadow: i32) -> Self {
        let shadow = if shadow == -1 { 0 } else { shadow as u32 };
        Style { base_colour: colour, colour, base_shadow: shadow, shadow, strike: None, underline: None, justify: 0, justify_acc: 0 }
    }

    /// The tags `method369` actually honours. Its `argb=`, `str=`, `u=` and `shad=` checks
    /// test the tag the wrong way round and so never fire; they are left out on purpose.
    fn apply(&mut self, tag: &str) {
        let alpha = self.colour & 0xff00_0000;
        if let Some(hex) = tag.strip_prefix("col=") {
            if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                self.colour = alpha | rgb & 0xff_ffff;
            }
        }
        match tag {
            "/col" => self.colour = alpha | self.base_colour & 0xff_ffff,
            "/argb" => self.colour = self.base_colour,
            "str" => self.strike = Some(alpha | 0x80_0000),
            "/str" => self.strike = None,
            "u" => self.underline = Some(alpha),
            "/u" => self.underline = None,
            "/shad" => self.shadow = self.base_shadow,
            "shad" => self.shadow = alpha,
            "br" => *self = Style::new(self.base_colour, self.base_shadow as i32),
            _ if tag.eq_ignore_ascii_case("shad=-1") => self.shadow = 0,
            _ => {}
        }
    }
}

impl Font {
    pub fn new(metrics: FontMetrics, glyphs: Vec<Sprite>) -> Self {
        Font { metrics, glyphs }
    }

    /// Port of `method371`: one line with its baseline at `y`.
    fn draw_line(&self, r: &mut Raster, text: &str, mut x: i32, y: i32, style: &mut Style) {
        let m = &self.metrics;
        let top = y - m.vertical_spacing;
        let mut previous: Option<u8> = None;
        for token in tokens(text) {
            let g = match token {
                Token::Tag(tag) => {
                    style.apply(&tag);
                    continue;
                }
                Token::Glyph(g) => g,
            };
            if let Some(p) = previous {
                x += m.kerning(p, g);
            }
            if g != b' ' {
                if let Some(glyph) = self.glyphs.get(g as usize) {
                    if style.shadow & 0xff00_0000 != 0 {
                        r.draw_glyph(glyph, x + 1, top + 1, style.shadow);
                    }
                    r.draw_glyph(glyph, x, top, style.colour);
                }
            } else if style.justify > 0 {
                style.justify_acc += style.justify;
                x += style.justify_acc >> 8;
                style.justify_acc &= 0xff;
            }
            let advance = m.advance(g);
            if let Some(c) = style.strike {
                r.horizontal_line(x, top + (0.7 * m.vertical_spacing as f64) as i32, advance, c);
            }
            if let Some(c) = style.underline {
                r.horizontal_line(x, top + m.vertical_spacing + 1, advance, c);
            }
            x += advance;
            previous = Some(g);
        }
    }

    /// Port of `method360`: one line centred on `centre_x`, baseline at `y`.
    pub fn draw_centred(&self, r: &mut Raster, text: &str, centre_x: i32, y: i32, colour: u32, shadow: i32) {
        let mut style = Style::new(colour, shadow);
        self.draw_line(r, text, centre_x - self.metrics.text_width(text) / 2, y, &mut style);
    }

    /// Port of `method373` (as called through `method378`, without a line limit):
    /// wraps text into the box and places the lines by the two alignments.
    #[allow(clippy::too_many_arguments)] // mirrors the client's text call: box, colours, alignments, line height
    pub fn draw_paragraph(
        &self,
        r: &mut Raster,
        text: &str,
        (x, y, w, h): (i32, i32, i32, i32),
        colour: u32,
        shadow: i32,
        h_align: HAlign,
        v_align: VAlign,
        line_height: i32,
    ) -> usize {
        let m = &self.metrics;
        let mut step = if line_height == 0 { m.vertical_spacing } else { line_height };
        // Boxes too small for two lines don't wrap.
        let wrap = !(h < step + m.bottom_padding + m.top_padding && h < step + step);
        let lines = m.split_lines(text, wrap.then_some(w));
        let n = lines.len() as i32;
        let v_align = if v_align == VAlign::Spread && n == 1 { VAlign::Centre } else { v_align };
        let mut line_y = match v_align {
            VAlign::Top => y + m.top_padding,
            VAlign::Centre => (h - m.top_padding - m.bottom_padding - step * (n - 1)) / 2 + y + m.top_padding,
            VAlign::Bottom => y + h - m.bottom_padding - step * (n - 1),
            VAlign::Spread => {
                let gap = ((h - m.top_padding - m.bottom_padding - step * (n - 1)) / (n + 1)).max(0);
                step += gap;
                y + m.top_padding + gap
            }
        };
        let mut style = Style::new(colour, shadow);
        for (i, line) in lines.iter().enumerate() {
            let line_x = match h_align {
                HAlign::Left => x,
                HAlign::Centre => x + (w - m.text_width(line)) / 2,
                HAlign::Right => x + w - m.text_width(line),
                HAlign::Justify => {
                    if i + 1 < lines.len() {
                        let gaps = tokens(line).iter().filter(|t| **t == Token::Glyph(b' ')).count() as i32;
                        if gaps > 0 {
                            style.justify = ((w - m.text_width(line)) << 8) / gaps;
                        }
                    }
                    x
                }
            };
            self.draw_line(r, line, line_x, line_y, &mut style);
            style.justify = 0;
            line_y += step;
        }
        lines.len()
    }
}
