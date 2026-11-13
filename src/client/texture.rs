//! A texture's pixels from its material (`MaterialDefinitions.renderIntPixels`), row by row:
//! each operation gives a row of one channel (`method12319`) or three (`getPixels`), values in
//! 0..4096, pulling its inputs' rows as it needs them (`method12317`, `method12333`). The
//! reference keeps recent rows in small caches (`Class320`, `Class308`); every row here is kept,
//! which gives the same pixels.
//!
//! Ported operations: 0 constant, 1 colour, 2 / 3 x / y, 5 blur, 6 clamp, 7 blend, 8 curve,
//! 10 colour ramp. A material using any other fails (`Unported`).

use crate::cache::material::{Material, Params};
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
    mono: HashMap<(usize, usize), Mono>,
    rgb: HashMap<(usize, usize), Rgb>,
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
            _ => return Err(Unported(self.m.ops[op].kind)),
        };
        let out = Rc::new(out);
        self.mono.insert((op, row), out.clone());
        Ok(out)
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

/// `MaterialDefinitions.renderIntPixels` at `size` × `size`: ARGB, each row written right to
/// left; a brick tile column by column.
pub fn render(m: &Material, size: usize, brick: bool) -> Result<Vec<u32>, Unported> {
    let axis: Vec<i32> = (0..size).map(|i| ((i as i32) << 12) / size as i32).collect();
    let mut r = Render { m, w: size, xs: axis.clone(), ys: axis, mono: HashMap::new(), rgb: HashMap::new() };
    let table = brightness_table(0.7);
    let [opaque, translucent, _] = m.outputs;
    let mut out = vec![0u32; size * size];
    let mut at = 0;
    for y in 0..size {
        let colour = if m.ops[opaque].mono {
            let v = r.mono(opaque, y)?;
            Rc::new([v.to_vec(), v.to_vec(), v.to_vec()])
        } else {
            r.rgb(opaque, y)?
        };
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

    fn op(kind: u8, inputs: Vec<usize>, mono: bool, params: Params) -> Op {
        Op { kind, inputs, image: -1, mono, params }
    }

    #[test]
    fn a_colour_over_a_constant_alpha() {
        // Opaque: colour (255, 0, 0); translucent: constant 4096.
        let m = Material {
            ops: vec![op(1, vec![], false, Params::Colour([255 << 4, 0, 0])), op(0, vec![], true, Params::Constant(4096))],
            outputs: [0, 1, 0],
        };
        let px = render(&m, 4, false).unwrap();
        assert!(px.iter().all(|&p| p == 0xffff_0000));
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
        let px = render(&m, 4, false).unwrap();
        assert_eq!(px[0] & 0xff, brightness_table(0.7)[(3 << 10 >> 4) as usize] as u32);
    }
}
