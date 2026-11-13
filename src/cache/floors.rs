//! Floor definitions (index 2): underlays (archive 1, `UnderlayDef`) and overlays (archive 4,
//! `OverlayDef`), file n = definition n. A map tile's underlay and overlay bytes name them.
//!
//! Kept: the colours and textures the ground is drawn with. Read past: shadow and blend flags,
//! texture scales and the water settings of overlays (opcodes 4, 5, 8..22).

use super::buffer::Reader;
use super::error::{corrupt, Result};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Underlay {
    /// `rgb` (opcode 1).
    pub rgb: i32,
    /// `texture` (opcode 2), -1 none.
    pub texture: i32,
    /// `method8048`'s split of `rgb`, what neighbouring tiles blend: `r` (hue × chroma), `g`
    /// (saturation), `b` (lightness), `a` (chroma, at least 1).
    pub hue_chroma: i32,
    pub saturation: i32,
    pub lightness: i32,
    pub chroma: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Overlay {
    /// `primaryRGB` (opcode 1) as packed HSL (`Class5.method294`), -1 for 0xff00ff (no colour).
    pub colour: i32,
    /// `texture` (opcodes 2, 3), -1 none.
    pub texture: i32,
    /// `secondaryRGB` (opcode 7) as packed HSL, -1 none.
    pub secondary: i32,
    /// `anInt7052`: which overlay wins where blending overlays meet, (opcode 11, default 8)
    /// << 8 | the definition's id (`method11356`).
    pub priority: i32,
    /// `aBool7061` (opcode 12): the overlay spreads into the tiles around it.
    pub blends: bool,
}

impl Underlay {
    /// `UnderlayDef.decode`.
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut u = Underlay { texture: -1, ..Default::default() };
        loop {
            match r.u8()? {
                0 => return Ok(u),
                1 => {
                    u.rgb = r.u24()? as i32;
                    u.split_colour();
                }
                2 => {
                    u.texture = match r.u16()? {
                        65535 => -1,
                        t => t as i32,
                    }
                }
                3 => {
                    r.u16()?; // scale
                }
                4 | 5 => {} // shadow and blend flags
                op => return corrupt(format!("unknown underlay opcode {op}")),
            }
        }
    }
}

impl Underlay {
    /// `UnderlayDef.method8048`.
    fn split_colour(&mut self) {
        let rgb = self.rgb;
        let (r, g, b) = (((rgb >> 16) & 0xff) as f64 / 256.0, ((rgb >> 8) & 0xff) as f64 / 256.0, (rgb & 0xff) as f64 / 256.0);
        let lo = r.min(g).min(b);
        let hi = if g > r { g } else { r };
        let hi = if b > hi { b } else { hi };
        let (mut hue, mut sat) = (0.0, 0.0);
        let light = (hi + lo) / 2.0;
        if hi != lo {
            if light < 0.5 {
                sat = (hi - lo) / (hi + lo);
            }
            if light >= 0.5 {
                sat = (hi - lo) / (2.0 - hi - lo);
            }
            if r == hi {
                hue = (g - b) / (hi - lo);
            } else if hi == g {
                hue = 2.0 + (b - r) / (hi - lo);
            } else if hi == b {
                hue = (r - g) / (hi - lo) + 4.0;
            }
        }
        hue /= 6.0;
        self.saturation = ((sat * 256.0) as i32).clamp(0, 255);
        self.lightness = ((256.0 * light) as i32).clamp(0, 255);
        self.chroma = if light > 0.5 { (sat * (1.0 - light) * 512.0) as i32 } else { (sat * light * 512.0) as i32 }.max(1);
        self.hue_chroma = (hue * self.chroma as f64) as i32;
    }
}

/// `Class371.method6348`: blended hue, saturation and lightness (0..=255 each) to packed HSL,
/// saturation lowered for very light colours as in `rgb_to_hsl`.
pub fn pack_hsl(hue: i32, mut saturation: i32, lightness: i32) -> i32 {
    saturation >>= match lightness {
        244.. => 4,
        218..=243 => 3,
        193..=217 => 2,
        180..=192 => 1,
        _ => 0,
    };
    (((hue & 0xff) >> 2) << 10) + (lightness >> 1) + ((saturation >> 5) << 7)
}

impl Overlay {
    /// `OverlayDef.method11357` for definition `id`, then `method11356`.
    pub fn decode(data: &[u8], id: u32) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut o = Overlay { colour: 0, texture: -1, secondary: -1, priority: 8, blends: false };
        loop {
            match r.u8()? {
                0 => {
                    o.priority = o.priority << 8 | id as i32;
                    return Ok(o);
                }
                1 => o.colour = rgb_to_overlay_hsl(r.u24()? as i32),
                2 => o.texture = r.u8()? as i32,
                3 => {
                    o.texture = match r.u16()? {
                        65535 => -1,
                        t => t as i32,
                    }
                }
                7 => o.secondary = rgb_to_overlay_hsl(r.u24()? as i32),
                5 | 8 | 10 => {}
                11 => o.priority = r.u8()? as i32,
                12 => o.blends = true,
                14 | 16 | 21 => {
                    r.u8()?;
                }
                9 | 20 | 22 => {
                    r.u16()?;
                }
                13 => {
                    r.u24()?;
                }
                op => return corrupt(format!("unknown overlay opcode {op}")),
            }
        }
    }
}

/// `IdentikitDefinition.method912`: 0xff00ff means no colour (-1).
fn rgb_to_overlay_hsl(rgb: i32) -> i32 {
    if rgb == 0xff00ff {
        -1
    } else {
        rgb_to_hsl(rgb)
    }
}

/// `Class5.method294`: 24-bit RGB to the client's packed HSL (6 bits hue, 3 saturation,
/// 7 lightness), saturation lowered for very light colours.
pub fn rgb_to_hsl(rgb: i32) -> i32 {
    let (r, g, b) = (((rgb >> 16) & 0xff) as f64 / 256.0, ((rgb >> 8) & 0xff) as f64 / 256.0, (rgb & 0xff) as f64 / 256.0);
    let lo = r.min(g).min(b);
    let hi = if g > r { g } else { r };
    let hi = if b > hi { b } else { hi };
    let (mut hue, mut sat) = (0.0, 0.0);
    let light = (hi + lo) / 2.0;
    if hi != lo {
        if light < 0.5 {
            sat = (hi - lo) / (lo + hi);
        }
        if light >= 0.5 {
            sat = (hi - lo) / (2.0 - hi - lo);
        }
        if r == hi {
            hue = (g - b) / (hi - lo);
        } else if g == hi {
            hue = 2.0 + (b - r) / (hi - lo);
        } else if hi == b {
            hue = 4.0 + (r - g) / (hi - lo);
        }
    }
    hue /= 6.0;
    let h = (256.0 * hue) as i32;
    let mut s = ((sat * 256.0) as i32).clamp(0, 255);
    let l = ((light * 256.0) as i32).clamp(0, 255);
    s >>= match l {
        244.. => 4,
        218..=243 => 3,
        193..=217 => 2,
        180..=192 => 1,
        _ => 0,
    };
    (((h & 0xff) >> 2) << 10) + (l >> 1) + ((s >> 5) << 7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_decode_their_colours() {
        let u = Underlay::decode(&[1, 0x40, 0x80, 0x20, 2, 0, 9, 0]).unwrap();
        assert_eq!((u.rgb, u.texture), (0x408020, 9));
        let o = Overlay::decode(&[1, 0xff, 0, 0xff, 7, 0, 0, 0, 13, 1, 2, 3, 0], 5).unwrap();
        assert_eq!((o.colour, o.secondary, o.texture, o.priority, o.blends), (-1, 0, -1, (8 << 8) | 5, false));
        let o = Overlay::decode(&[11, 3, 12, 0], 2).unwrap();
        assert_eq!((o.priority, o.blends), ((3 << 8) | 2, true));
        // Pure red: hue 0, full saturation, lightness 128 → saturation 7 (top 3 bits), l 64.
        assert_eq!(rgb_to_hsl(0xff0000), (7 << 7) + 63);
    }
}
