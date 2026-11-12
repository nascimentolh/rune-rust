//! A software framebuffer with a clip rectangle, like the reference client's
//! Java renderer: everything is drawn into ARGB pixels, then shown as one texture.

use crate::cache::sprite::Sprite;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    pub left: i32,
    pub top: i32,
    /// Exclusive.
    pub right: i32,
    /// Exclusive.
    pub bottom: i32,
}

pub struct Raster {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u32>,
    clip: Clip,
}

/// Blends `src` (ARGB) over `dst` (RGB) using the source alpha.
fn blend(dst: u32, src: u32) -> u32 {
    let a = src >> 24;
    match a {
        0 => dst,
        255 => src & 0xff_ffff,
        _ => {
            let inv = 255 - a;
            let rb = ((src & 0xff_00ff) * a + (dst & 0xff_00ff) * inv) >> 8 & 0xff_00ff;
            let g = ((src & 0x00_ff00) * a + (dst & 0x00_ff00) * inv) >> 8 & 0x00_ff00;
            rb | g
        }
    }
}

impl Raster {
    pub fn new(width: i32, height: i32) -> Self {
        let full = Clip { left: 0, top: 0, right: width, bottom: height };
        Raster { width, height, pixels: vec![0; (width * height) as usize], clip: full }
    }

    pub fn clear(&mut self, rgb: u32) {
        self.pixels.fill(rgb & 0xff_ffff);
    }

    pub fn clip(&self) -> Clip {
        self.clip
    }

    /// Port of `r(x1, y1, x2, y2)`: sets the clip, limited to the raster.
    pub fn set_clip(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        self.clip = Clip { left: left.max(0), top: top.max(0), right: right.min(self.width), bottom: bottom.min(self.height) };
    }

    pub fn restore_clip(&mut self, clip: Clip) {
        self.clip = clip;
    }

    fn plot(&mut self, x: i32, y: i32, argb: u32) {
        let c = self.clip;
        if x >= c.left && x < c.right && y >= c.top && y < c.bottom {
            let i = (y * self.width + x) as usize;
            self.pixels[i] = blend(self.pixels[i], argb);
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, argb: u32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.plot(xx, yy, argb);
            }
        }
    }

    /// AWT-style `drawRect`: the outline covers w + 1 by h + 1 pixels.
    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, argb: u32) {
        self.fill_rect(x, y, w + 1, 1, argb);
        self.fill_rect(x, y + h, w + 1, 1, argb);
        self.fill_rect(x, y, 1, h + 1, argb);
        self.fill_rect(x + w, y, 1, h + 1, argb);
    }

    pub fn horizontal_line(&mut self, x: i32, y: i32, w: i32, argb: u32) {
        self.fill_rect(x, y, w, 1, argb);
    }

    /// Draws a sprite with its top-left frame corner at (x, y).
    pub fn draw_sprite(&mut self, s: &Sprite, x: i32, y: i32) {
        let (ox, oy) = (x + s.min_x, y + s.min_y);
        for row in 0..s.height {
            for col in 0..s.width {
                self.plot(ox + col as i32, oy + row as i32, s.pixels[row * s.width + col]);
            }
        }
    }

    /// Port of `method2772`: repeats the sprite from (x, y) to fill w × h, within the clip.
    pub fn draw_sprite_tiled(&mut self, s: &Sprite, x: i32, y: i32, w: i32, h: i32) {
        let saved = self.clip;
        self.clip = Clip { right: saved.right.min(x + w), bottom: saved.bottom.min(y + h), ..saved };
        let (sw, sh) = (s.width as i32 + s.min_x, s.height as i32 + s.min_y);
        if sw > 0 && sh > 0 {
            for ty in 0..(h + sh - 1) / sh {
                for tx in 0..(w + sw - 1) / sw {
                    self.draw_sprite(s, x + tx * sw, y + ty * sh);
                }
            }
        }
        self.clip = saved;
    }

    /// Draws a font glyph as a mask: each pixel's alpha scaled by the colour's alpha.
    pub fn draw_glyph(&mut self, glyph: &Sprite, x: i32, y: i32, argb: u32) {
        let colour_alpha = argb >> 24;
        let (ox, oy) = (x + glyph.min_x, y + glyph.min_y);
        for row in 0..glyph.height {
            for col in 0..glyph.width {
                let mask = glyph.pixels[row * glyph.width + col] >> 24;
                if mask != 0 {
                    let a = mask * colour_alpha / 255;
                    self.plot(ox + col as i32, oy + row as i32, a << 24 | argb & 0xff_ffff);
                }
            }
        }
    }

    /// Mixes `other` over this raster: alpha 0 keeps this, 255 shows `other`.
    pub fn mix_from(&mut self, other: &Raster, alpha: u32) {
        for (dst, &src) in self.pixels.iter_mut().zip(&other.pixels) {
            *dst = blend(*dst, alpha.min(255) << 24 | src);
        }
    }

    /// RGBA bytes for a texture.
    pub fn to_rgba(&self) -> Vec<u8> {
        self.pixels.iter().flat_map(|&p| [(p >> 16) as u8, (p >> 8) as u8, p as u8, 255]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_limits_drawing_and_tiles_repeat() {
        let mut r = Raster::new(8, 4);
        let dot = Sprite { min_x: 0, min_y: 0, width: 2, height: 1, pixels: vec![0xffff0000, 0] };
        r.set_clip(0, 0, 5, 4);
        r.draw_sprite_tiled(&dot, 0, 0, 8, 1);
        assert_eq!(&r.pixels[0..8], &[0xff0000, 0, 0xff0000, 0, 0xff0000, 0, 0, 0]);
    }

    #[test]
    fn half_alpha_blends() {
        assert_eq!(blend(0x000000, 0x80ff_ffff) & 0xff, 0x7f); // (255 * 128) >> 8, as the client computes it
    }
}
