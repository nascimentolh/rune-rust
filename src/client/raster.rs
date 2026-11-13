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

/// Blends `src` (ARGB) over `dst` using the source alpha. `dst`'s top byte is how much of what
/// lies behind the raster still shows through (0 normally; 255 where `clear_see_through` left
/// it untouched), and shrinks as colour is laid over it.
fn blend(dst: u32, src: u32) -> u32 {
    let a = src >> 24;
    match a {
        0 => dst,
        255 => src & 0xff_ffff,
        _ => {
            let inv = 255 - a;
            let rb = ((src & 0xff_00ff) * a + (dst & 0xff_00ff) * inv) >> 8 & 0xff_00ff;
            let g = ((src & 0x00_ff00) * a + (dst & 0x00_ff00) * inv) >> 8 & 0x00_ff00;
            let through = (dst >> 24) * inv / 255;
            through << 24 | rb | g
        }
    }
}

/// `method2742`'s colour mode 0: channels multiplied by the tint's (`c * t >> 8`), alpha by
/// the tint's alpha. A white tint only changes the alpha.
fn tinted(p: u32, tint: u32) -> u32 {
    let alpha = (p >> 24) * (tint >> 24) / 255;
    let rgb = if tint & 0xff_ffff == 0xff_ffff {
        p & 0xff_ffff
    } else {
        let channel = |shift: u32| (((p >> shift) & 0xff) * ((tint >> shift) & 0xff)) >> 8 << shift;
        channel(16) | channel(8) | channel(0)
    };
    alpha << 24 | rgb
}

impl Raster {
    pub fn new(width: i32, height: i32) -> Self {
        let full = Clip { left: 0, top: 0, right: width, bottom: height };
        Raster { width, height, pixels: vec![0; (width * height) as usize], clip: full }
    }

    pub fn clear(&mut self, rgb: u32) {
        self.pixels.fill(rgb & 0xff_ffff);
    }

    /// Clears to nothing, so what is drawn behind the raster shows where nothing is drawn.
    pub fn clear_see_through(&mut self) {
        self.pixels.fill(0xff00_0000);
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

    /// Port of `method2772`: repeats the sprite's frame from (x, y) to fill w × h, within the clip.
    pub fn draw_sprite_tiled(&mut self, s: &Sprite, x: i32, y: i32, w: i32, h: i32, tint: Option<u32>) {
        let saved = self.clip;
        self.clip = Clip { right: saved.right.min(x + w), bottom: saved.bottom.min(y + h), ..saved };
        let (sw, sh) = (s.frame_width, s.frame_height);
        if sw > 0 && sh > 0 {
            for ty in 0..(h + sh - 1) / sh {
                for tx in 0..(w + sw - 1) / sw {
                    self.draw_sprite_scaled(s, x + tx * sw, y + ty * sh, sw, sh, tint);
                }
            }
        }
        self.clip = saved;
    }

    /// Draws the sprite's whole frame stretched to w × h at (x, y) (`method2755`; at the
    /// frame's own size it is `method2742`). With a tint, each colour channel is multiplied
    /// by the tint's and the alpha by the tint's alpha (colour mode 0).
    ///
    /// ponytail: nearest-pixel sampling; the reference steps in 16.16 fixed point, which can
    /// pick a neighbouring pixel at some scales.
    pub fn draw_sprite_scaled(&mut self, s: &Sprite, x: i32, y: i32, w: i32, h: i32, tint: Option<u32>) {
        let (fw, fh) = (s.frame_width, s.frame_height);
        if w <= 0 || h <= 0 || fw <= 0 || fh <= 0 {
            return;
        }
        for dy in 0..h {
            let sy = dy * fh / h - s.min_y;
            if sy < 0 || sy >= s.height as i32 {
                continue;
            }
            for dx in 0..w {
                let sx = dx * fw / w - s.min_x;
                if sx < 0 || sx >= s.width as i32 {
                    continue;
                }
                let p = s.pixels[sy as usize * s.width + sx as usize];
                if p != 0 {
                    self.plot(x + dx, y + dy, tint.map_or(p, |t| tinted(p, t)));
                }
            }
        }
    }

    /// A one-pixel line (`method8433`), every point opaque in `rgb`.
    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, rgb: u32) {
        let steps = (x2 - x1).abs().max((y2 - y1).abs());
        for i in 0..=steps {
            let (x, y) = if steps == 0 { (x1, y1) } else { (x1 + (x2 - x1) * i / steps, y1 + (y2 - y1) * i / steps) };
            self.plot(x, y, 0xff00_0000 | rgb);
        }
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

    /// A flat triangle, filled row by row (the software toolkit's `method2425`, flat-shaded).
    pub fn fill_triangle(&mut self, a: (i32, i32), b: (i32, i32), c: (i32, i32), argb: u32) {
        let mut v = [a, b, c];
        v.sort_by_key(|p| p.1);
        let [(x0, y0), (x1, y1), (x2, y2)] = v;
        let edge = |xa: i32, ya: i32, xb: i32, yb: i32, y: i32| if yb == ya { xa } else { xa + (xb - xa) * (y - ya) / (yb - ya) };
        for y in y0..y2 {
            let long = edge(x0, y0, x2, y2, y);
            let short = if y < y1 { edge(x0, y0, x1, y1, y) } else { edge(x1, y1, x2, y2, y) };
            let (l, r) = (long.min(short), long.max(short));
            self.fill_rect(l, y, r - l, 1, argb);
        }
    }

    pub fn save_png(&self, path: &std::path::Path) -> Result<(), String> {
        let img = image::RgbaImage::from_raw(self.width as u32, self.height as u32, self.to_rgba()).ok_or("raster size mismatch")?;
        img.save(path).map_err(|e| e.to_string())
    }

    /// RGBA bytes for a texture: opaque, except where `clear_see_through` left gaps.
    pub fn to_rgba(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|&p| {
                let alpha = 255 - (p >> 24);
                // Blending onto the gaps leaves colour already weighted by its alpha.
                let channel = |c: u32| ((c & 0xff) * 255).checked_div(alpha).map_or(0, |v| v.min(255) as u8);
                [channel(p >> 16), channel(p >> 8), channel(p), alpha as u8]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_limits_drawing_and_tiles_repeat() {
        let mut r = Raster::new(8, 4);
        let dot = Sprite { min_x: 0, min_y: 0, frame_width: 2, frame_height: 1, width: 2, height: 1, pixels: vec![0xffff0000, 0] };
        r.set_clip(0, 0, 5, 4);
        r.draw_sprite_tiled(&dot, 0, 0, 8, 1, None);
        assert_eq!(&r.pixels[0..8], &[0xff0000, 0, 0xff0000, 0, 0xff0000, 0, 0, 0]);
    }

    #[test]
    fn half_alpha_blends() {
        assert_eq!(blend(0x000000, 0x80ff_ffff) & 0xff, 0x7f); // (255 * 128) >> 8, as the client computes it
    }
}
