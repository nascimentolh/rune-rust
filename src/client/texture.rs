//! A texture's pixels from its material (`MaterialDefinitions.renderIntPixels`), row by row:
//! each operation gives a row of one channel (`method12319`) or three (`getPixels`), values in
//! 0..4096, pulling its inputs' rows as it needs them (`method12317`, `method12333`). The
//! reference keeps recent rows in small caches (`Class320`, `Class308`); every row here is kept,
//! which gives the same pixels.
//!
//! Ported operations: 0 constant, 1 colour, 2 / 3 x / y, 5 blur, 6 clamp, 7 blend, 8 curve, 18 / 39
//! a sprite (repeated / stretched), 36 another texture, 4 bricks, 11 tint, 12 waves, 13 position hash, 20 tiles, 26 threshold, 27 stripes, 29 shapes, 35 slopes, 38 scratches,
//! 9 mirror, 10 colour ramp, 15 cells, 17 HSL shift, 19 warp, 21 mix, 22 invert, 23 kaleidoscope, 30 range, 32 emboss, 34 noise. A
//! material using any other fails (`Unported`).

use crate::cache::material::{cos_table, sin_table, Cells, JavaRandom, Material, Noise, Params, Scratches};
use crate::cache::sprite::Sprite;
use crate::client::shapes;
use std::collections::HashMap;
use std::rc::Rc;

type Mono = Rc<Vec<i32>>;
type Rgb = Rc<[Vec<i32>; 3]>;

struct Render<'a> {
    m: &'a Material,
    w: usize,
    /// `Class316.anIntArray3672` / `anIntArray3668`: x and y as 0..4096.
    xs: Vec<i32>,
    ys: Vec<i32>,
    /// `Class316.anIntArray3675` / `anIntArray3677`.
    cos: [i32; 256],
    sin: [i32; 256],
    emboss: Vec<u8>,
    /// `Class370.anIntArray4291`: the noise fade curve over 0..4096.
    fade: Vec<i32>,
    /// Operation 17's colour left by its last pixel (`anInt9927`, `anInt9921`, `anInt9924`).
    hsl_last: HashMap<usize, [i32; 3]>,
    mono: HashMap<(usize, usize), Mono>,
    rgb: HashMap<(usize, usize), Rgb>,
    /// The pictures operations 18, 36 and 39 read (`method12315`), by operation; `None` for a
    /// sprite operation with no sprite, which leaves its rows 0.
    pictures: HashMap<usize, Option<Picture>>,
    /// Operation 29's and 38's whole picture, drawn the first time a row is asked for.
    canvases: HashMap<usize, Rc<Vec<Vec<i32>>>>,
}

/// What a texture may read besides its own material.
pub trait Images {
    /// Texture `id`'s material, and whether it is half size and a brick tile
    /// (`ImageIndexLoader.getMaterialDefinitions`, `getTextureDetails`).
    fn texture(&mut self, id: i32) -> Option<(Material, bool, bool)>;
    /// Sprite `id` (`SpriteDefinitions.method1515`: the first sprite of index 8's file `id`).
    fn sprite(&mut self, id: i32) -> Option<Sprite>;
}

/// An RGB picture an operation reads, row by row.
struct Picture {
    w: usize,
    h: usize,
    px: Vec<u32>,
}

impl Picture {
    /// `SpriteDefinitions.load` then `getPixels`: the image in its full frame, 0 around it.
    fn of_sprite(s: &Sprite) -> Picture {
        let (w, h) = (s.frame_width.max(0) as usize, s.frame_height.max(0) as usize);
        let mut px = vec![0; w * h];
        for y in 0..s.height {
            for x in 0..s.width {
                let (fx, fy) = (x + s.min_x as usize, y + s.min_y as usize);
                if fx < w && fy < h {
                    px[fy * w + fx] = s.pixels[y * s.width + x] & 0xff_ffff;
                }
            }
        }
        Picture { w, h, px }
    }
}

/// Colours as three channels × 16 (`(c & 0xff0000) >> 12`, `(c & 0xff00) >> 4`, `(c & 0xff) << 4`).
fn channels(px: &[u32]) -> [Vec<i32>; 3] {
    [
        px.iter().map(|&c| (c as i32 & 0xff0000) >> 12).collect(),
        px.iter().map(|&c| (c as i32 & 0xff00) >> 4).collect(),
        px.iter().map(|&c| (c as i32 & 0xff) << 4).collect(),
    ]
}

/// How deep textures may read other textures; the reference has no limit (a texture that
/// reads itself overflows its stack), so deeper ones are refused.
const MAX_DEPTH: u8 = 8;

/// `MaterialProp32.aByteArray9814`: 255 / √((y² + x² + 65535) / 65535) for 0 ≤ y ≤ x < 256, at
/// x × (x + 1) / 2 + y.
fn emboss_table() -> Vec<u8> {
    let mut t = Vec::with_capacity(32896);
    for x in 0..256i32 {
        for y in 0..=x {
            let f = (y * y + x * x + 65535) as f32 / 65535.0;
            t.push((255.0 / (f as f64).sqrt()) as i32 as u8);
        }
    }
    t
}

/// `Class370.method6326`: 6t⁵ − 15t⁴ + 10t³ in 12-bit fixed point.
fn fade(t: i32) -> i32 {
    let cube = (t * ((t * t) >> 12)) >> 12;
    let a = t * 6 - 61440;
    let b = ((t * a) >> 12) + 40960;
    (b * cube) >> 12
}

/// `MaterialProp34.method15198`: gradient noise at `x` (20.12 fixed point) on a row whose
/// fraction is `y` with cell hashes `a` and `b` (this and the next cell down), `fade_y` its
/// fade, wrapping at `wrap` cells across.
fn gradient(perm: &[u8; 512], fade: &[i32], x: i32, y: i32, (a, b): (usize, usize), fade_y: i32, wrap: i32) -> i32 {
    let cell = x >> 12;
    let next = if cell + 1 >= wrap { 0 } else { cell + 1 };
    let x = x & 0xfff;
    let (cell, next) = ((cell & 0xff) as usize, (next & 0xff) as usize);
    let (x1, y1) = (x - 4096, y - 4096);
    let fx = fade[x as usize];
    // The four corner gradients `& 0x3`: (1, 1), (−1, 1), (1, −1), (−1, −1).
    let g = |h: u8, dx: i32, dy: i32| match h & 3 {
        0 => dy + dx,
        1 => dy - dx,
        2 => dx - dy,
        _ => -dx - dy,
    };
    let (g0, g1) = (g(perm[a + cell], x, y), g(perm[a + next], x1, y));
    let top = g0 + (fx.wrapping_mul(g1 - g0) >> 12);
    let (g2, g3) = (g(perm[cell + b], x, y1), g(perm[next + b], x1, y1));
    let bottom = g2 + (fx.wrapping_mul(g3 - g2) >> 12);
    top + (fade_y.wrapping_mul(bottom - top) >> 12)
}

/// `MaterialProp13.method15428`: 4096 − an integer hash of the position (x, y in 0..4096),
/// / 262144.
fn hash(x: i32, y: i32) -> i32 {
    let mut n = y.wrapping_mul(57).wrapping_add(x);
    n ^= n.wrapping_shl(1);
    let h = n.wrapping_mul(n).wrapping_mul(15731).wrapping_add(789221).wrapping_mul(n).wrapping_add(1376312589);
    4096 - (h & 0x7fff_ffff) / 262144
}

/// `MaterialProp15.method12319`: one distance of the four kept per pixel.
fn cell_distance(mode: i32, dx: i32, dy: i32) -> i32 {
    let root = |v: i32| ((v.wrapping_abs() as f32 / 4096.0) as f64).sqrt();
    match mode {
        1 => dy.wrapping_mul(dy).wrapping_add(dx.wrapping_mul(dx)) >> 12,
        2 => dx.wrapping_abs().wrapping_add(dy.wrapping_abs()),
        3 => dx.wrapping_abs().max(dy.wrapping_abs()),
        4 => {
            let s = ((root(dx) * 4096.0) as i32).wrapping_add((root(dy) * 4096.0) as i32);
            s.wrapping_mul(s) >> 12
        }
        5 => {
            let sum = dy.wrapping_mul(dy).wrapping_add(dx.wrapping_mul(dx));
            (((sum as f32 / 1.6777216E7) as f64).sqrt().sqrt() * 4096.0) as i32
        }
        _ => {
            let sum = dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
            (((sum as f32 / 1.6777216E7) as f64).sqrt() * 4096.0) as i32
        }
    }
}

/// `MaterialProp17.method15400`: RGB (0..4096) to (hue, saturation, lightness).
fn to_hsl(r: i32, g: i32, b: i32) -> [i32; 3] {
    let (hi, lo) = (r.max(g).max(b), r.min(g).min(b));
    let d = hi - lo;
    let l = (lo + hi) / 2;
    let s = if l > 0 && l < 4096 { (d << 12) / if l <= 2048 { l * 2 } else { 8192 - l * 2 } } else { 0 };
    let h = if d > 0 {
        let (cr, cg, cb) = (((hi - r) << 12) / d, ((hi - g) << 12) / d, ((hi - b) << 12) / d);
        let h = if hi == r {
            if lo == g {
                cb + 20480
            } else {
                4096 - cg
            }
        } else if hi == g {
            if b == lo {
                cr + 4096
            } else {
                12288 - cb
            }
        } else if lo == r {
            cg + 12288
        } else {
            20480 - cr
        };
        h / 6
    } else {
        0
    };
    [h, s, l]
}

/// `MaterialProp17.method15401`: (hue, saturation, lightness) back to RGB; `None` for a hue
/// of 4096, which leaves the previous pixel's colour (the switch has no case for it).
fn from_hsl(h: i32, s: i32, l: i32) -> Option<[i32; 3]> {
    let q = if l <= 2048 { l.wrapping_mul(s + 4096) >> 12 } else { l + s - (l.wrapping_mul(s) >> 12) };
    if q <= 0 {
        // The reference's blue here is l × −680551435 (a deobfuscation leftover); it is only
        // reached with l = 0.
        return Some([l, l, l.wrapping_mul(-680551435)]);
    }
    let h6 = h.wrapping_mul(6);
    let p = l + l - q;
    let f = ((q - p) << 12) / q;
    let sector = h6 >> 12;
    let frac = h6 - (sector << 12);
    let t = frac.wrapping_mul(f.wrapping_mul(q) >> 12) >> 12;
    let (up, down) = (p + t, q - t);
    match sector {
        0 => Some([q, up, p]),
        1 => Some([down, q, p]),
        2 => Some([p, q, up]),
        3 => Some([p, down, q]),
        4 => Some([up, p, q]),
        5 => Some([q, p, down]),
        _ => None,
    }
}

/// `MaterialProp17.getPixels` for one pixel: shifted by `shift` (hue, saturation, lightness);
/// `last` is the colour the operation left before (its fields start at 0).
fn shift_hsl(rgb: [i32; 3], shift: [i32; 3], last: &mut [i32; 3]) -> [i32; 3] {
    let [mut h, mut s, mut l] = to_hsl(rgb[0], rgb[1], rgb[2]);
    h += shift[0];
    s += shift[1];
    l += shift[2];
    while h < 0 {
        h += 4096;
    }
    while h > 4096 {
        h -= 4096;
    }
    if let Some(c) = from_hsl(h, s.clamp(0, 4096), l.clamp(0, 4096)) {
        *last = c;
    }
    *last
}

/// Why a texture can't be drawn yet.
#[derive(Debug, PartialEq)]
pub struct Unported(pub u8);

impl Render<'_> {
    /// `method12317`: input `k` of operation `op` as one channel (a coloured one gives red).
    fn in_mono(&mut self, op: usize, k: usize, row: usize) -> Result<Mono, Unported> {
        let i = self.m.ops[op].inputs[k];
        if self.m.ops[i].mono {
            self.mono(i, row)
        } else {
            Ok(Rc::new(self.rgb(i, row)?[0].clone()))
        }
    }

    /// `method12333`: input `k` as three channels (a one-channel input three times).
    fn in_rgb(&mut self, op: usize, k: usize, row: usize) -> Result<Rgb, Unported> {
        let i = self.m.ops[op].inputs[k];
        if self.m.ops[i].mono {
            let v = self.mono(i, row)?;
            Ok(Rc::new([v.to_vec(), v.to_vec(), v.to_vec()]))
        } else {
            self.rgb(i, row)
        }
    }

    /// `method12319` of operation `op` for `row`.
    fn mono(&mut self, op: usize, row: usize) -> Result<Mono, Unported> {
        if let Some(r) = self.mono.get(&(op, row)) {
            return Ok(r.clone());
        }
        let w = self.w;
        let out: Vec<i32> = match &self.m.ops[op].params {
            Params::Constant(v) => vec![*v; w],
            Params::Horizontal => self.xs.clone(),
            Params::Vertical => vec![self.ys[row]; w],
            Params::Blur(rx, ry) => {
                let (rx, ry) = (*rx, *ry);
                let mut rows = Vec::new();
                for y in row as i32 - ry..=row as i32 + ry {
                    let src = self.in_mono(op, 0, self.wrap_y(y))?;
                    rows.push(blur_row(&src, rx, w));
                }
                let f = 65536 / (ry + ry + 1);
                (0..w).map(|x| rows.iter().map(|r| r[x]).fold(0i32, |a, v| a.wrapping_add(v)).wrapping_mul(f) >> 16).collect()
            }
            Params::Clamp(lo, hi) => {
                let (lo, hi) = (*lo, *hi);
                self.in_mono(op, 0, row)?.iter().map(|&v| if v < lo { lo } else { v.min(hi) }).collect()
            }
            Params::Blend(mode) => {
                let mode = *mode;
                let (a, b) = (self.in_mono(op, 0, row)?, self.in_mono(op, 1, row)?);
                (0..w).map(|x| blend(mode, a[x], b[x])).collect()
            }
            Params::Curve(table) => {
                let table = table.clone();
                self.in_mono(op, 0, row)?.iter().map(|&v| table[(v >> 4).clamp(0, 256) as usize] as i32).collect()
            }
            Params::Mirror(mx, my) => {
                let (mx, my) = (*mx, *my);
                let src = self.in_mono(op, 0, if my { self.ys.len() - 1 - row } else { row })?;
                (0..w).map(|x| src[if mx { w - 1 - x } else { x }]).collect()
            }
            Params::Warp(strength) => {
                let strength = *strength;
                let (angle, amount) = (self.in_mono(op, 1, row)?, self.in_mono(op, 2, row)?);
                let mut out = vec![0; w];
                for (x, o) in out.iter_mut().enumerate() {
                    // `>> 4 & 0xff` for one channel.
                    let (sx, sy) = self.warp(x, row, (angle[x] >> 4) & 0xff, strength, amount[x]);
                    *o = self.in_mono(op, 0, sy)?[sx];
                }
                out
            }
            Params::Mix => {
                let (a, b, t) = (self.in_mono(op, 0, row)?, self.in_mono(op, 1, row)?, self.in_mono(op, 2, row)?);
                (0..w).map(|x| mix(a[x], b[x], t[x], false)).collect()
            }
            Params::Invert => self.in_mono(op, 0, row)?.iter().map(|&v| 4096 - v).collect(),
            Params::Range(lo, span) => {
                let (lo, span) = (*lo, *span);
                self.in_mono(op, 0, row)?.iter().map(|&v| (span.wrapping_mul(v) >> 12) + lo).collect()
            }
            Params::Emboss(scale, light) => {
                let (scale, light) = (*scale, *light);
                let k = scale.wrapping_mul(w as i32 * 32) >> 12;
                let below = self.in_mono(op, 0, self.wrap_y(row as i32 - 1))?;
                let here = self.in_mono(op, 0, row)?;
                let above = self.in_mono(op, 0, self.wrap_y(row as i32 + 1))?;
                let mask = w as i32 - 1;
                (0..w)
                    .map(|x| {
                        let xi = x as i32;
                        let dy = (above[x] - below[x]).wrapping_mul(k) >> 12;
                        let dx = k.wrapping_mul(here[((xi - 1) & mask) as usize] - here[((xi + 1) & mask) as usize]) >> 12;
                        let (a, b) = (((dx >> 4).abs()).min(255), ((dy >> 4).abs()).min(255));
                        let n = self.emboss[(a + ((b * (b + 1)) >> 1)) as usize] as i32;
                        let (nx, ny, nz) = (dx.wrapping_mul(n) >> 8, dy.wrapping_mul(n) >> 8, (n * 4096) >> 8);
                        (nz.wrapping_mul(light[2]) >> 12) + (light[1].wrapping_mul(ny) >> 12) + (nx.wrapping_mul(light[0]) >> 12)
                    })
                    .collect()
            }
            // One channel holds the shapes' RGB colours as they are.
            Params::Shapes(_) | Params::Scratches(_) => self.canvas(op)?[row].clone(),
            Params::Kaleidoscope => {
                let mut last = (0, 0);
                let mut out = vec![0; w];
                for (x, o) in out.iter_mut().enumerate() {
                    let (sx, sy) = self.kaleidoscope(x, row, &mut last);
                    *o = self.in_mono(op, 0, sy)?[sx];
                }
                out
            }
            Params::Waves { round, wave, frequency } => {
                // `MaterialProp12.method12319`.
                let (round, wave, frequency) = (*round, *wave, *frequency);
                let y = self.ys[row];
                let half_y = (y - 2048) >> 1;
                self.xs
                    .iter()
                    .map(|&x| {
                        let mut v = if round {
                            let half_x = (x - 2048) >> 1;
                            let d = half_y.wrapping_mul(half_y).wrapping_add(half_x.wrapping_mul(half_x)) >> 12;
                            let r = (((d as f32 / 4096.0) as f64).sqrt() * 4096.0) as i32;
                            (r.wrapping_mul(frequency) as f64 * std::f64::consts::PI) as i32
                        } else {
                            (x - y).wrapping_mul(frequency)
                        };
                        // The position within its 4096 wave.
                        v -= v & -4096;
                        match wave {
                            0 => (self.sin[((v >> 4) & 0xff) as usize] + 4096) >> 1,
                            2 => (2048 - (v - 2048).abs()) << 1,
                            _ => v,
                        }
                    })
                    .collect()
            }
            Params::Bricks(b) => {
                // `MaterialProp4.method12319`: wrapped into 0..=4096, the row (and odd or even),
                // then the brick; mortar within half its width of an edge is 0.
                let wrap = |mut v: i32| {
                    while v < 0 {
                        v += 4096;
                    }
                    while v > 4096 {
                        v -= 4096;
                    }
                    v
                };
                let y = wrap(self.ys[row] + b.y_shift);
                let count = b.rows.len() - 1;
                let i = (0..count).take_while(|&i| y >= b.rows[i]).count();
                let r = i.checked_sub(1).ok_or(Unported(4))?;
                if y > b.rows[r] + b.mortar && y < b.rows[i] - b.mortar {
                    let shift = if i & 1 == 0 { b.offset } else { -b.offset };
                    let starts = &b.starts[r];
                    let columns = starts.len() - 1;
                    self.xs
                        .iter()
                        .map(|&x| {
                            let x = wrap(x + (shift.wrapping_mul(b.brick_width) >> 12));
                            let j = (0..columns).take_while(|&j| x >= starts[j]).count();
                            let c = j.saturating_sub(1);
                            if j > 0 && x > starts[c] + b.mortar && x < starts[j] - b.mortar {
                                b.shades[r][c]
                            } else {
                                0
                            }
                        })
                        .collect()
                } else {
                    vec![0; w]
                }
            }
            Params::Stripes(st) => {
                // `MaterialProp27.method12319`: each position's stripe, on before its end.
                let on = |v: i32| -> i32 {
                    let i = (0..st.starts.len() - 1).find(|&i| v >= st.starts[i] && v < st.starts[i + 1]);
                    i.map_or(0, |i| if v < st.ends[i] { 4096 } else { 0 })
                };
                let y = self.ys[row];
                if st.direction == 0 {
                    vec![on(y); w]
                } else {
                    self.xs
                        .iter()
                        .map(|&x| {
                            on(match st.direction {
                                1 => x,
                                2 => ((x - (4096 - y)) >> 1) + 2048,
                                3 => ((x - y) >> 1) + 2048,
                                _ => 0,
                            })
                        })
                        .collect()
                }
            }
            Params::Threshold(lo, hi) => {
                let (lo, hi) = (*lo, *hi);
                self.in_mono(op, 0, row)?.iter().map(|&v| if v >= lo && v <= hi { 4096 } else { 0 }).collect()
            }
            Params::Slopes(strength) => {
                // `MaterialProp35`: the rows above and below and the pixels either side (wrapping).
                let strength = *strength;
                let above = self.in_mono(op, 0, self.wrap_y(row as i32 - 1))?;
                let here = self.in_mono(op, 0, row)?;
                let below = self.in_mono(op, 0, self.wrap_y(row as i32 + 1))?;
                let mask = w as i32 - 1;
                (0..w)
                    .map(|x| {
                        let xi = x as i32;
                        let dy = below[x].wrapping_sub(above[x]).wrapping_mul(strength) >> 12;
                        let dx =
                            here[((xi + 1) & mask) as usize].wrapping_sub(here[((xi - 1) & mask) as usize]).wrapping_mul(strength) >> 12;
                        let sum = (dy.wrapping_mul(dy) >> 12).wrapping_add(dx.wrapping_mul(dx) >> 12).wrapping_add(4096);
                        let len = (((sum as f32 / 4096.0) as f64).sqrt() * 4096.0) as i32;
                        4096 - if len != 0 { 16777216 / len } else { 0 }
                    })
                    .collect()
            }
            Params::Hash => {
                let y = self.ys[row];
                self.xs.iter().map(|&x| hash(x, y) % 4096).collect()
            }
            Params::Noise(n) => self.noise(n, row),
            Params::Cells(c) => self.cells(c, row),
            _ => return Err(Unported(self.m.ops[op].kind)),
        };
        let out = Rc::new(out);
        self.mono.insert((op, row), out.clone());
        Ok(out)
    }

    /// `MaterialProp38.method12319`: `count` scratches of `length` pixels from random points at
    /// random angles, each a Bresenham line along its longer axis (the far end left out), its
    /// value rising from a random start by 2048 / its length per pixel; written at row
    /// x & (width - 1), column y & (height - 1) (the reference's indexing, row from x). `None`
    /// where the reference would write outside its arrays.
    fn scratches(&self, s: &Scratches) -> Option<Vec<Vec<i32>>> {
        let (w, h) = (self.w as i32, self.ys.len() as i32);
        let mut c = vec![vec![0; self.w]; self.ys.len()];
        let mut random = JavaRandom::new(s.seed as i64);
        let half = s.spread >> 1;
        for _ in 0..s.count {
            let a = if s.spread > 0 { s.angle + (random.below(s.spread) - half) } else { s.angle };
            let a = ((a >> 4) & 0xff) as usize;
            let (mut x0, mut y0) = (random.below(w), random.below(h));
            let (mut x1, mut y1) = (x0 + (self.cos[a].wrapping_mul(s.length) >> 12), y0 + (self.sin[a].wrapping_mul(s.length) >> 12));
            let (dy, dx) = ((y1 - y0).abs(), (x1 - x0).abs());
            if dx == 0 && dy == 0 {
                continue;
            }
            let steep = dy > dx;
            if steep {
                (x0, y0, x1, y1) = (y0, x0, y1, x1);
            }
            if x0 > x1 {
                (x0, y0, x1, y1) = (x1, y1, x0, y0);
            }
            let (run, rise) = (x1 - x0, (y1 - y0).abs());
            let mut err = -run / 2;
            let step = 2048 / run;
            let start = 1024 - (random.below(4096) >> 2);
            let dir = if y0 < y1 { 1 } else { -1 };
            let mut y = y0;
            for x in x0..x1 {
                let v = start + step * (x - x0) + 1024;
                // `i_23 & anInt3669`, `i_16 & anInt3673`.
                let (major, minor) = ((x & (w - 1)) as usize, (y & (h - 1)) as usize);
                let (r, col) = if steep { (minor, major) } else { (major, minor) };
                *c.get_mut(r)?.get_mut(col)? = v;
                err += rise;
                if err > 0 {
                    err -= run;
                    y += dir;
                }
            }
        }
        Some(c)
    }

    /// `MaterialProp29.method15372`: the shapes drawn over the whole picture at once
    /// (`Class320.method5722`, `Class308.method5464`), which needs a row cache that keeps
    /// every row (the reference throws otherwise).
    fn canvas(&mut self, op: usize) -> Result<Rc<Vec<Vec<i32>>>, Unported> {
        if let Some(c) = self.canvases.get(&op) {
            return Ok(c.clone());
        }
        let (m, kind, rows) = (self.m, self.m.ops[op].kind, self.m.ops[op].rows);
        if rows != 255 && rows as usize != self.ys.len() {
            return Err(Unported(kind));
        }
        let drawn = match &m.ops[op].params {
            Params::Shapes(list) => shapes::draw(list, self.w, self.ys.len()).ok(),
            Params::Scratches(s) => self.scratches(s),
            _ => None,
        };
        let c = Rc::new(drawn.ok_or(Unported(kind))?);
        self.canvases.insert(op, c.clone());
        Ok(c)
    }

    /// `MaterialProp34.method15197`: each octave's noise added at its strength; an octave of
    /// strength −8..=8 is skipped when there are several. The reference then keeps whatever its
    /// row cache held; that is 0 here.
    fn noise(&self, n: &Noise, row: usize) -> Vec<i32> {
        let (cx, cy) = n.cells;
        let y = self.ys[row].wrapping_mul(cy);
        let mut out = vec![0i32; self.w];
        let count = n.amplitude.len();
        for (i, (&amp, &freq)) in n.amplitude.iter().zip(&n.frequency).enumerate() {
            let amp = amp as i32;
            if count > 1 && (-8..=8).contains(&amp) {
                continue;
            }
            let f = (freq as i32) << 12;
            let (wrap_y, wrap_x) = (f.wrapping_mul(cy) >> 12, f.wrapping_mul(cx) >> 12);
            let fy = y.wrapping_mul(f) >> 12;
            let cell = fy >> 12;
            let next = if cell + 1 >= wrap_y { 0 } else { cell + 1 };
            let fy = fy & 0xfff;
            let hashes = (n.perm[(cell & 0xff) as usize] as usize, n.perm[(next & 0xff) as usize] as usize);
            let fade_y = self.fade[fy as usize];
            let last = n.centred && i == count - 1;
            for (o, &x) in out.iter_mut().zip(&self.xs) {
                let v = gradient(&n.perm, &self.fade, f.wrapping_mul(x.wrapping_mul(cx)) >> 12, fy, hashes, fade_y, wrap_x);
                *o = o.wrapping_add(amp.wrapping_mul(v) >> 12);
                if last {
                    *o = (*o >> 1) + 2048;
                }
            }
        }
        out
    }

    /// `MaterialProp15.method12319`: the distances from each pixel to the jittered points of
    /// the 3 × 3 cells around it, the four smallest kept.
    fn cells(&self, c: &Cells, row: usize) -> Vec<i32> {
        let (cx, cy) = c.cells;
        let y = self.ys[row].wrapping_mul(cy) + 2048;
        let cell_y = y >> 12;
        let wrap = |v: i32, n: i32| if v >= n { v - n } else { v };
        self.xs
            .iter()
            .map(|&x| {
                // Nearest first (`Class368.anInt4266`, `IgnoredPlayer.anInt118`,
                // `Ground.anInt4777`, `RemoveMemberSetting.anInt10003`).
                let mut best = [i32::MAX; 4];
                let x = x.wrapping_mul(cx) + 2048;
                let cell_x = x >> 12;
                for j in cell_y - 1..=cell_y + 1 {
                    let h = c.perm[(wrap(j, cy) & 0xff) as usize] as i32;
                    for i in cell_x - 1..=cell_x + 1 {
                        let k = 2 * c.perm[((h + wrap(i, cx)) & 0xff) as usize] as usize;
                        let dx = x - (c.offsets[k] as i32 + (i << 12));
                        let dy = y - (c.offsets[k + 1] as i32 + (j << 12));
                        let d = cell_distance(c.distance, dx, dy);
                        if let Some(at) = best.iter().position(|&b| d < b) {
                            best.copy_within(at..3, at + 1);
                            best[at] = d;
                        }
                    }
                }
                match c.output {
                    0 => best[0],
                    1 => best[1],
                    2 => best[1].wrapping_sub(best[0]),
                    3 => best[2],
                    4 => best[3],
                    // No case: the row cache's old value, 0 here.
                    _ => 0,
                }
            })
            .collect()
    }

    /// `MaterialProp23.method15383`: where pixel (`x`, `row`) reads from, by the angle of its
    /// offset from the centre (atan2 as a float) in eighths of a turn, masked to the picture.
    /// An angle outside every eighth (a float π, straight above the centre) keeps the source
    /// the pixel before it used (the reference keeps it in static fields).
    fn kaleidoscope(&self, x: usize, row: usize, last: &mut (usize, usize)) -> (usize, usize) {
        use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
        const THREE_PI_4: f64 = 2.356194490192345;
        let (w, h) = (self.w as i32, self.ys.len() as i32);
        let (xi, yi) = (x as i32, row as i32);
        let f = ((self.xs[x] - 2048) as f64).atan2((self.ys[row] - 2048) as f64) as f32;
        let d = f as f64;
        let pick = if (-PI..=-THREE_PI_4).contains(&d) {
            (xi, yi)
        } else if (-THREE_PI_4..=-FRAC_PI_2).contains(&d) {
            (yi, xi)
        } else if (-FRAC_PI_2..=-FRAC_PI_4).contains(&d) {
            (w - yi, xi)
        } else if f <= 0.0 && d >= -FRAC_PI_4 {
            (xi, h - yi)
        } else if f >= 0.0 && d <= FRAC_PI_4 {
            (w - xi, h - yi)
        } else if (FRAC_PI_4..=FRAC_PI_2).contains(&d) {
            (w - yi, h - xi)
        } else if (FRAC_PI_2..=THREE_PI_4).contains(&d) {
            (yi, h - xi)
        } else if (THREE_PI_4..=PI).contains(&d) {
            (w - xi, yi)
        } else {
            return *last;
        };
        *last = ((pick.0 & (w - 1)) as usize, (pick.1 & (h - 1)) as usize);
        *last
    }

    /// `MaterialProp19`: where pixel (`x`, `row`) reads its source, `angle` 0..256 of a turn and
    /// `amount` × `strength` >> 12 the distance.
    fn warp(&self, x: usize, row: usize, angle: i32, strength: i32, amount: i32) -> (usize, usize) {
        let d = strength.wrapping_mul(amount) >> 12;
        let dx = self.cos[angle as usize].wrapping_mul(d) >> 12;
        let dy = self.sin[angle as usize].wrapping_mul(d) >> 12;
        let sx = (x as i32 + (dx >> 12)) & (self.w as i32 - 1);
        let sy = ((dy >> 12) + row as i32) & (self.ys.len() as i32 - 1);
        (sx as usize, sy as usize)
    }

    /// `getPixels` of operation `op` for `row`.
    fn rgb(&mut self, op: usize, row: usize) -> Result<Rgb, Unported> {
        if let Some(r) = self.rgb.get(&(op, row)) {
            return Ok(r.clone());
        }
        let w = self.w;
        let out: [Vec<i32>; 3] = match &self.m.ops[op].params {
            Params::Colour(c) => c.map(|v| vec![v; w]),
            Params::Blur(rx, ry) => {
                let (rx, ry) = (*rx, *ry);
                let mut rows = Vec::new();
                for y in row as i32 - ry..=row as i32 + ry {
                    let src = self.in_rgb(op, 0, self.wrap_y(y))?;
                    rows.push([0, 1, 2].map(|c| blur_row(&src[c], rx, w)));
                }
                let f = 65536 / (ry + ry + 1);
                [0, 1, 2].map(|c| {
                    (0..w).map(|x| rows.iter().map(|r| r[c][x]).fold(0i32, |a, v| a.wrapping_add(v)).wrapping_mul(f) >> 16).collect()
                })
            }
            Params::Clamp(lo, hi) => {
                let (lo, hi) = (*lo, *hi);
                let src = self.in_rgb(op, 0, row)?;
                [0, 1, 2].map(|c| src[c].iter().map(|&v| if v < lo { lo } else { v.min(hi) }).collect())
            }
            Params::Blend(mode) => {
                let mode = *mode;
                let (a, b) = (self.in_rgb(op, 0, row)?, self.in_rgb(op, 1, row)?);
                [0, 1, 2].map(|c| (0..w).map(|x| blend(mode, a[c][x], b[c][x])).collect())
            }
            Params::Ramp(table) => {
                let table = table.clone();
                let src = self.in_mono(op, 0, row)?;
                let rgb: Vec<i32> = src.iter().map(|&v| table[(v >> 4).clamp(0, 256) as usize]).collect();
                [
                    rgb.iter().map(|c| (c & 0xff0000) >> 12).collect(),
                    rgb.iter().map(|c| (c & 0xff00) >> 4).collect(),
                    rgb.iter().map(|c| (c & 0xff) << 4).collect(),
                ]
            }
            Params::Mirror(mx, my) => {
                let (mx, my) = (*mx, *my);
                let src = self.in_rgb(op, 0, if my { self.ys.len() - 1 - row } else { row })?;
                [0, 1, 2].map(|c| (0..w).map(|x| src[c][if mx { w - 1 - x } else { x }]).collect())
            }
            Params::Hsl(shift) => {
                let shift = *shift;
                let src = self.in_rgb(op, 0, row)?;
                let mut last = self.hsl_last.get(&op).copied().unwrap_or([0; 3]);
                let px: Vec<[i32; 3]> = (0..w).map(|x| shift_hsl([src[0][x], src[1][x], src[2][x]], shift, &mut last)).collect();
                self.hsl_last.insert(op, last);
                [0, 1, 2].map(|c| px.iter().map(|p| p[c]).collect())
            }
            Params::Warp(strength) => {
                let strength = *strength;
                let (angle, amount) = (self.in_mono(op, 1, row)?, self.in_mono(op, 2, row)?);
                let mut out = [vec![0; w], vec![0; w], vec![0; w]];
                for x in 0..w {
                    // `× 255 >> 12 & 0xff` for three channels.
                    let (sx, sy) = self.warp(x, row, (angle[x].wrapping_mul(255) >> 12) & 0xff, strength, amount[x]);
                    let src = self.in_rgb(op, 0, sy)?;
                    for c in 0..3 {
                        out[c][x] = src[c][sx];
                    }
                }
                out
            }
            Params::Mix => {
                let t = self.in_mono(op, 2, row)?;
                let (a, b) = (self.in_rgb(op, 0, row)?, self.in_rgb(op, 1, row)?);
                [0, 1, 2].map(|c| (0..w).map(|x| mix(a[c][x], b[c][x], t[x], true)).collect())
            }
            Params::Invert => {
                let src = self.in_rgb(op, 0, row)?;
                [0, 1, 2].map(|c| src[c].iter().map(|&v| 4096 - v).collect())
            }
            Params::Range(lo, span) => {
                let (lo, span) = (*lo, *span);
                let src = self.in_rgb(op, 0, row)?;
                [0, 1, 2].map(|c| src[c].iter().map(|&v| (span.wrapping_mul(v) >> 12) + lo).collect())
            }
            Params::Shapes(_) => {
                let c = self.canvas(op)?;
                channels(&c[row].iter().map(|&v| v as u32).collect::<Vec<_>>())
            }
            Params::Tint(tint) => {
                // `MaterialProp11.getPixels`: only a grey (all three equal) keeps its level.
                let tint = *tint;
                let src = self.in_rgb(op, 0, row)?;
                let grey = |x: usize| src[0][x] == src[2][x] && src[1][x] == src[2][x];
                [0, 1, 2].map(|k| (0..w).map(|x| if grey(x) { src[0][x].wrapping_mul(tint[k]) >> 12 } else { tint[k] }).collect())
            }
            Params::Kaleidoscope => {
                let mut last = (0, 0);
                let mut px = Vec::with_capacity(w);
                for x in 0..w {
                    let (sx, sy) = self.kaleidoscope(x, row, &mut last);
                    let src = self.in_rgb(op, 0, sy)?;
                    px.push([src[0][sx], src[1][sx], src[2][sx]]);
                }
                [0, 1, 2].map(|k| px.iter().map(|p| p[k]).collect())
            }
            Params::Tile(across, down) => {
                // `MaterialProp20.getPixels`: tiles of width / across by height / down, each the
                // whole input shrunk (a count of 0 divides by zero in the reference).
                let (across, down) = (*across, *down);
                if across == 0 || down == 0 {
                    return Err(Unported(20));
                }
                let height = self.ys.len();
                let (tile_w, tile_h) = (w / across as usize, height / down as usize);
                let src_row = (row % tile_h.max(1) * height).checked_div(tile_h).unwrap_or(0);
                let src = self.in_rgb(op, 0, src_row)?;
                let at = |x: usize| (x % tile_w.max(1) * w).checked_div(tile_w).unwrap_or(0);
                [0, 1, 2].map(|c| (0..w).map(|x| src[c][at(x)]).collect())
            }
            Params::Texture | Params::Sprite => match self.pictures.get(&op) {
                // `getPixels`: row h·y / height (y when the heights match), column x·w / width
                // (x when the widths match).
                Some(Some(p)) => {
                    let height = self.ys.len();
                    let start = if p.h != height { p.h * row / height } else { row } * p.w;
                    let at = |x: usize| start + if p.w != w { x * p.w / w } else { x };
                    let row: Option<Vec<u32>> = (0..w).map(|x| p.px.get(at(x)).copied()).collect();
                    channels(&row.ok_or(Unported(self.m.ops[op].kind))?)
                }
                Some(None) => [vec![0; w], vec![0; w], vec![0; w]],
                None => return Err(Unported(self.m.ops[op].kind)),
            },
            Params::TiledSprite => match self.pictures.get(&op) {
                // `MaterialPropSpriteSub18`: the row starts at (y % h) × h (h, not w, as the
                // reference has it), then x % w.
                Some(Some(p)) if p.w > 0 && p.h > 0 => {
                    let start = row % p.h * p.h;
                    let row: Option<Vec<u32>> = (0..w).map(|x| p.px.get(x % p.w + start).copied()).collect();
                    channels(&row.ok_or(Unported(18))?)
                }
                Some(None) => [vec![0; w], vec![0; w], vec![0; w]],
                _ => return Err(Unported(18)),
            },
            _ => return Err(Unported(self.m.ops[op].kind)),
        };
        let out = Rc::new(out);
        self.rgb.insert((op, row), out.clone());
        Ok(out)
    }

    /// Rows wrap (`& Class316.anInt3673`).
    fn wrap_y(&self, y: i32) -> usize {
        (y & (self.ys.len() as i32 - 1)) as usize
    }
}

/// `MaterialProp5`'s horizontal pass: the running sum over x ± `r` (wrapping), × 65536 / (2r + 1).
fn blur_row(src: &[i32], r: i32, w: usize) -> Vec<i32> {
    let mask = w as i32 - 1;
    let f = 65536 / (r + r + 1);
    let mut sum = (-r..=r).fold(0i32, |a, i| a.wrapping_add(src[(i & mask) as usize]));
    let mut out = vec![0; w];
    for (x, o) in out.iter_mut().enumerate() {
        *o = sum.wrapping_mul(f) >> 16;
        let x = x as i32;
        sum = sum.wrapping_sub(src[((x - r) & mask) as usize]).wrapping_add(src[((x + 1 + r) & mask) as usize]);
    }
    out
}

/// `MaterialProp21`: `a` where `t` is 4096, `b` where it is 0, else between (the one-channel
/// and three-channel sums are written in a different order, kept as each is).
fn mix(a: i32, b: i32, t: i32, rgb: bool) -> i32 {
    match t {
        4096 => a,
        0 => b,
        _ if rgb => (t.wrapping_mul(a).wrapping_add((4096 - t).wrapping_mul(b))) >> 12,
        _ => ((4096 - t).wrapping_mul(b).wrapping_add(t.wrapping_mul(a))) >> 12,
    }
}

/// `MaterialProp7`: inputs `a` and `b` combined by `mode` (any other mode leaves 0).
fn blend(mode: i32, a: i32, b: i32) -> i32 {
    let div = |n: i32, d: i32| n.wrapping_div(d);
    match mode {
        1 => a.wrapping_add(b),
        2 => a.wrapping_sub(b),
        3 => a.wrapping_mul(b) >> 12,
        4 => {
            if b != 0 {
                div(a.wrapping_shl(12), b)
            } else {
                4096
            }
        }
        5 => 4096 - ((4096 - a).wrapping_mul(4096 - b) >> 12),
        6 => {
            if b < 2048 {
                b.wrapping_mul(a) >> 11
            } else {
                4096 - ((4096 - a).wrapping_mul(4096 - b) >> 11)
            }
        }
        7 => {
            if a == 4096 {
                4096
            } else {
                div(b.wrapping_shl(12), 4096 - a)
            }
        }
        8 => {
            if a == 0 {
                0
            } else {
                4096 - div((4096 - b).wrapping_shl(12), a)
            }
        }
        9 => a.min(b),
        10 => a.max(b),
        11 => (a - b).abs(),
        12 => b.wrapping_add(a).wrapping_sub(a.wrapping_mul(b) >> 11),
        _ => 0,
    }
}

/// `LRUCache.method3895`: each channel raised to `power` (0.7 for textures).
fn brightness_table(power: f64) -> [i32; 256] {
    std::array::from_fn(|i| ((((i as f64) / 255.0).powf(power) * 255.0) as i32).min(255))
}

/// `method12315` of the picture operations: each 36 renders its texture (at 64 when half
/// size, else 128), each 39 and 18 loads its sprite.
fn pictures(m: &Material, images: &mut dyn Images, depth: u8) -> Result<HashMap<usize, Option<Picture>>, Unported> {
    let mut out = HashMap::new();
    for (i, o) in m.ops.iter().enumerate() {
        match o.params {
            Params::Texture => {
                // No texture (−1) leaves the pixels null, which the reference can't draw.
                let found = if o.image >= 0 && depth < MAX_DEPTH { images.texture(o.image) } else { None };
                let (inner, half, brick) = found.ok_or(Unported(36))?;
                let size = if half { 64 } else { 128 };
                let px = pixels(&inner, size, brick, images, depth + 1)?;
                out.insert(i, Some(Picture { w: size, h: size, px }));
            }
            Params::Sprite | Params::TiledSprite if o.image < 0 => {
                out.insert(i, None);
            }
            Params::Sprite | Params::TiledSprite => {
                let sprite = images.sprite(o.image).ok_or(Unported(o.kind))?;
                out.insert(i, Some(Picture::of_sprite(&sprite)));
            }
            _ => {}
        }
    }
    Ok(out)
}

impl<'a> Render<'a> {
    fn new(m: &'a Material, size: usize, images: &mut dyn Images, depth: u8) -> Result<Self, Unported> {
        let pictures = pictures(m, images, depth)?;
        let axis: Vec<i32> = (0..size).map(|i| ((i as i32) << 12) / size as i32).collect();
        Ok(Render {
            m,
            w: size,
            xs: axis.clone(),
            ys: axis,
            cos: cos_table(),
            sin: sin_table(),
            emboss: emboss_table(),
            fade: (0..4096).map(fade).collect(),
            hsl_last: HashMap::new(),
            mono: HashMap::new(),
            rgb: HashMap::new(),
            pictures,
            canvases: HashMap::new(),
        })
    }

    /// The opaque output's row as three channels.
    fn colour(&mut self, y: usize) -> Result<Rgb, Unported> {
        let opaque = self.m.outputs[0];
        if self.m.ops[opaque].mono {
            let v = self.mono(opaque, y)?;
            Ok(Rc::new([v.to_vec(), v.to_vec(), v.to_vec()]))
        } else {
            self.rgb(opaque, y)
        }
    }
}

/// `MaterialDefinitions.method14718` at brightness 1.0, as operation 36 renders another
/// texture: the opaque output only, rows left to right (a brick tile column by column), alpha
/// 255 wherever the colour isn't black.
fn pixels(m: &Material, size: usize, brick: bool, images: &mut dyn Images, depth: u8) -> Result<Vec<u32>, Unported> {
    let mut r = Render::new(m, size, images, depth)?;
    let table = brightness_table(1.0);
    let mut out = vec![0u32; size * size];
    let mut at = 0;
    for y in 0..size {
        let colour = r.colour(y)?;
        if brick {
            at = y;
        }
        for x in 0..size {
            let c = |v: i32| table[(v >> 4).clamp(0, 255) as usize];
            let rgb = (c(colour[2][x]) + (c(colour[1][x]) << 8) + (c(colour[0][x]) << 16)) as u32;
            out[at] = if rgb != 0 { rgb | 0xff00_0000 } else { 0 };
            at += 1;
            if brick {
                at += size - 1;
            }
        }
    }
    Ok(out)
}

/// `MaterialDefinitions.renderIntPixels` at `size` × `size`: ARGB, each row written right to
/// left; a brick tile column by column.
pub fn render(m: &Material, size: usize, brick: bool, images: &mut dyn Images) -> Result<Vec<u32>, Unported> {
    let mut r = Render::new(m, size, images, 0)?;
    let table = brightness_table(0.7);
    let translucent = m.outputs[1];
    let mut out = vec![0u32; size * size];
    let mut at = 0;
    for y in 0..size {
        let colour = r.colour(y)?;
        let alpha = if m.ops[translucent].mono { r.mono(translucent, y)? } else { Rc::new(r.rgb(translucent, y)?[0].clone()) };
        if brick {
            at = y;
        }
        for x in (0..size).rev() {
            let c = |v: i32| table[(v >> 4).clamp(0, 255) as usize];
            let (red, green, blue) = (c(colour[0][x]), c(colour[1][x]), c(colour[2][x]));
            let a = if red == 0 && green == 0 && blue == 0 { 0 } else { (alpha[x] >> 4).clamp(0, 255) };
            out[at] = (blue + (green << 8) + (a << 24) + (red << 16)) as u32;
            at += 1;
            if brick {
                at += size - 1;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::material::Op;

    /// No other textures or sprites.
    struct NoImages;

    impl Images for NoImages {
        fn texture(&mut self, _: i32) -> Option<(Material, bool, bool)> {
            None
        }

        fn sprite(&mut self, _: i32) -> Option<Sprite> {
            None
        }
    }

    fn op(kind: u8, inputs: Vec<usize>, mono: bool, params: Params) -> Op {
        Op { kind, inputs, image: -1, mono, rows: 255, params }
    }

    #[test]
    fn a_colour_over_a_constant_alpha() {
        // Opaque: colour (255, 0, 0); translucent: constant 4096.
        let m = Material {
            ops: vec![op(1, vec![], false, Params::Colour([255 << 4, 0, 0])), op(0, vec![], true, Params::Constant(4096))],
            outputs: [0, 1, 0],
        };
        let px = render(&m, 4, false, &mut NoImages).unwrap();
        assert!(px.iter().all(|&p| p == 0xffff_0000));
    }

    #[test]
    fn flat_input_has_no_slopes_and_a_threshold_keeps_its_range() {
        // Slopes of a constant: the normal is straight up (length 4096), so 4096 − 4096 = 0.
        let flat = |params| Material {
            ops: vec![
                op(0, vec![], true, Params::Constant(2048)),
                op(35, vec![0], true, params),
                op(0, vec![], true, Params::Constant(4096)),
            ],
            outputs: [1, 2, 0],
        };
        let px = render(&flat(Params::Slopes(4096)), 4, false, &mut NoImages).unwrap();
        assert!(px.iter().all(|&p| p == 0));
        // 2048 is inside 1024..=3072: white.
        let px = render(&flat(Params::Threshold(1024, 3072)), 4, false, &mut NoImages).unwrap();
        assert!(px.iter().all(|&p| p == 0xffff_ffff));
    }

    #[test]
    fn straight_waves_start_on_the_diagonal() {
        // At x = y the straight wave's position is 0: a sine gives 2048, a triangle 0.
        let waves = |wave| Material {
            ops: vec![
                op(12, vec![], true, Params::Waves { round: false, wave, frequency: 1 }),
                op(0, vec![], true, Params::Constant(4096)),
            ],
            outputs: [0, 1, 0],
        };
        let (sine, triangle) = (waves(0), waves(2));
        let mut r = Render::new(&sine, 4, &mut NoImages, 0).unwrap();
        assert_eq!(r.mono(0, 0).unwrap()[0], 2048);
        let mut r = Render::new(&triangle, 4, &mut NoImages, 0).unwrap();
        assert_eq!(r.mono(0, 0).unwrap()[0], 0);
    }

    #[test]
    fn the_position_hash_keeps_java_ints() {
        // At (0, 0) only the constant is left: 4096 − 1376312589 / 262144.
        assert_eq!(hash(0, 0), 4096 - 5250);
        // Its remainder keeps the sign, as Java's does.
        assert_eq!(hash(0, 0) % 4096, -1154);
    }

    #[test]
    fn blends_and_blurs_follow_the_reference() {
        assert_eq!((blend(3, 2048, 2048), blend(4, 1, 0), blend(11, 10, 30), blend(13, 1, 1)), (1024, 4096, 20, 0));
        // A blur of radius 1 over one 4096 among zeros spreads a third each way.
        let row = blur_row(&[4096, 0, 0, 0], 1, 4);
        assert_eq!(row, vec![(4096 * (65536 / 3)) >> 16, (4096 * (65536 / 3)) >> 16, 0, (4096 * (65536 / 3)) >> 16]);
        // x runs right to left in each row: the x ramp's first pixel is its last value.
        let m = Material {
            ops: vec![op(2, vec![], true, Params::Horizontal), op(0, vec![], true, Params::Constant(4096))],
            outputs: [0, 1, 0],
        };
        let px = render(&m, 4, false, &mut NoImages).unwrap();
        assert_eq!(px[0] & 0xff, brightness_table(0.7)[(3 << 10 >> 4) as usize] as u32);
    }
}
