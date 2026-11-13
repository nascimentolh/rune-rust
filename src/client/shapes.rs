//! Texture operation 29's shapes (`MaterialProp29.method15372`), drawn into a width × height
//! picture of RGB colours that starts black. The clip is the whole picture
//! (`WallDecoration.method16086`: 0..=width − 1, 0..=height − 1). Each shape scales its 0..4096
//! numbers by the size (`× size >> 12`) and goes through the reference's own routine, the
//! unclipped one when it fits and the clipped one when it doesn't, since the two differ by a
//! pixel here and there.
//!
//! Rectangles and ellipses always carry both colours (they're read as u24, never negative),
//! so they always take the fill-and-outline routine (`method2557`); lines and curves have no
//! fill and take the outline one (`method2561`).
//!
//! The reference writes past the picture's edge in a few broken cases (an unclipped routine
//! fed a negative size); that throws there and fails here (`Outside`).

use crate::cache::material::Shape;
use std::num::Wrapping;

/// A write the reference would make outside its arrays (it throws).
#[derive(Debug)]
pub struct Outside;

type Draw = Result<(), Outside>;

/// `Class532.anIntArrayArray7072` with its clip.
pub struct Canvas {
    pub rows: Vec<Vec<i32>>,
    max_x: i32,
    max_y: i32,
    /// `Class5.anIntArray36`: the inner circle's x per row. The reference keeps one array for
    /// every circle it ever draws; a fresh one per circle here.
    inner: Vec<i32>,
}

/// `EntityNode.method4890`: `v` clamped to `lo..=hi`.
fn clamp(v: i32, lo: i32, hi: i32) -> i32 {
    if v < lo {
        lo
    } else {
        v.min(hi)
    }
}

/// Draws `shapes` at `w` × `h`.
pub fn draw(shapes: &[Shape], w: usize, h: usize) -> Result<Vec<Vec<i32>>, Outside> {
    let mut c = Canvas { rows: vec![vec![0; w]; h], max_x: w as i32 - 1, max_y: h as i32 - 1, inner: Vec::new() };
    let (w, h) = (w as i32, h as i32);
    let sx = |v: i32| v.wrapping_mul(w) >> 12;
    let sy = |v: i32| h.wrapping_mul(v) >> 12;
    for s in shapes {
        match *s {
            Shape::Line { from, to, colour } => c.line(sx(from.0), sy(from.1), sx(to.0), sy(to.1), colour)?,
            Shape::Curve { points, colour } => c.curve(points.map(|(x, y)| (sx(x), sy(y))), colour)?,
            Shape::Rect { x, y, fill, outline, width } => c.rect((sx(x.0), sx(x.1)), (sy(y.0), sy(y.1)), fill, outline, width)?,
            Shape::Ellipse { centre, radius, fill, outline, width } => {
                c.ellipse((sx(centre.0), sy(centre.1)), (sx(radius.0), sy(radius.1)), fill, outline, width)?
            }
        }
    }
    Ok(c.rows)
}

impl Canvas {
    /// `KeyHoldInputSubscriber.method3922`: row `y` from `x1` to `x2` inclusive (nothing when
    /// `x1 > x2`; the row itself is looked up either way).
    fn span(&mut self, y: i32, x1: i32, x2: i32, c: i32) -> Draw {
        let row = usize::try_from(y).ok().and_then(|y| self.rows.get_mut(y)).ok_or(Outside)?;
        if x1 > x2 {
            return Ok(());
        }
        let (a, b) = (usize::try_from(x1).map_err(|_| Outside)?, x2 as usize);
        row.get_mut(a..=b).ok_or(Outside)?.fill(c);
        Ok(())
    }

    fn set(&mut self, y: i32, x: i32, c: i32) -> Draw {
        let row = usize::try_from(y).ok().and_then(|y| self.rows.get_mut(y)).ok_or(Outside)?;
        *usize::try_from(x).ok().and_then(|x| row.get_mut(x)).ok_or(Outside)? = c;
        Ok(())
    }

    fn inside_x(&self, x: i32) -> bool {
        x >= 0 && x <= self.max_x
    }

    fn inside_y(&self, y: i32) -> bool {
        y >= 0 && y <= self.max_y
    }

    /// `GroundItemStrategy.method13411`: column `x` from the lower y up to, not including, the
    /// higher.
    fn column(&mut self, x: i32, y1: i32, y2: i32, c: i32) -> Draw {
        for y in y1.min(y2)..y1.max(y2) {
            self.set(y, x, c)?;
        }
        Ok(())
    }

    /// `Class16.method568`: row `y` between `x1` and `x2`, either way round.
    fn row(&mut self, x1: i32, x2: i32, y: i32, c: i32) -> Draw {
        self.span(y, x1.min(x2), x1.max(x2), c)
    }

    /// `AnimationIndexLoader.method11220`: a line, unclipped (Bresenham, steep lines walked
    /// along y; straight ones as a column or row).
    fn bresenham(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: i32) -> Draw {
        let (dy, dx) = (y1.wrapping_sub(y0), x1.wrapping_sub(x0));
        if dx == 0 {
            return if dy != 0 { self.column(x0, y0, y1, c) } else { Ok(()) };
        }
        if dy == 0 {
            return self.row(x0, x1, y0, c);
        }
        let steep = dy.wrapping_abs() > dx.wrapping_abs();
        let (mut a, mut b) = if steep { ((y0, x0), (y1, x1)) } else { ((x0, y0), (x1, y1)) };
        if a.0 > b.0 {
            std::mem::swap(&mut a, &mut b);
        }
        let mut y = a.1;
        let run = b.0.wrapping_sub(a.0);
        let rise = b.1.wrapping_sub(a.1).wrapping_abs();
        let mut err = -(run >> 1);
        let step = if a.1 < b.1 { 1 } else { -1 };
        for x in a.0..=b.0 {
            if steep {
                self.set(x, y, c)?;
            } else {
                self.set(y, x, c)?;
            }
            err = err.wrapping_add(rise);
            if err > 0 {
                y += step;
                err = err.wrapping_sub(run);
            }
        }
        Ok(())
    }

    /// `QuickChatMessage.method6159`: a line clipped to the picture first (along its slope in
    /// 20.12 fixed point), then drawn unclipped.
    fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: i32) -> Draw {
        let (dx, dy) = (x1.wrapping_sub(x0), y1.wrapping_sub(y0));
        if dx == 0 {
            // `IdleAnimationsPreference.method12746`.
            if dy != 0 && self.inside_x(x0) {
                return self.column(x0, clamp(y0, 0, self.max_y), clamp(y1, 0, self.max_y), c);
            }
            return Ok(());
        }
        if dy == 0 {
            // `CS2Executor.method11250`.
            if self.inside_y(y0) {
                return self.row(clamp(x0, 0, self.max_x), clamp(x1, 0, self.max_x), y0, c);
            }
            return Ok(());
        }
        let slope = dy.wrapping_shl(12).wrapping_div(dx);
        let base = y0.wrapping_sub(x0.wrapping_mul(slope) >> 12);
        let on = |x: i32| (slope.wrapping_mul(x) >> 12).wrapping_add(base);
        // A slope of 0 divides by zero in the reference.
        let x_of = |y: i32| if slope == 0 { Err(Outside) } else { Ok(y.wrapping_sub(base).wrapping_shl(12).wrapping_div(slope)) };
        let ends = |x: i32, y: i32| -> Result<(i32, i32), Outside> {
            let (mut x, mut y) = if x < 0 {
                (0, on(0))
            } else if x > self.max_x {
                (self.max_x, on(self.max_x))
            } else {
                (x, y)
            };
            if y < 0 {
                (x, y) = (x_of(0)?, 0);
            } else if y > self.max_y {
                (x, y) = (x_of(self.max_y)?, self.max_y);
            }
            Ok((x, y))
        };
        let ((xa, ya), (xb, yb)) = (ends(x0, y0)?, ends(x1, y1)?);
        self.bresenham(xa, ya, xb, yb, c)
    }

    /// `Node_Sub24.method12399`: a cubic curve in 32 straight pieces (t every 128 of 4096);
    /// unclipped pieces (`ClanChannel.method12117`) when all four points are inside, clipped
    /// ones (`Engine.method4779`) otherwise. A curve whose ends are its control points is one
    /// line.
    fn curve(&mut self, p: [(i32, i32); 4], c: i32) -> Draw {
        let inside = p.iter().all(|&(x, y)| self.inside_x(x) && self.inside_y(y));
        let piece = |s: &mut Self, a: (i32, i32), b: (i32, i32)| {
            if inside {
                s.bresenham(a.0, a.1, b.0, b.1, c)
            } else {
                s.line(a.0, a.1, b.0, b.1, c)
            }
        };
        let [(x0, y0), (x1, y1), (x2, y2), (x3, y3)] = p.map(|(x, y)| (Wrapping(x), Wrapping(y)));
        if x0 == x1 && y1 == y0 && x2 == x3 && y2 == y3 {
            return piece(self, p[0], p[3]);
        }
        let three = Wrapping(3);
        let (ax, ay) = (x3 - three * x2 + three * x1 - x0, three * y1 + (y3 - three * y2) - y0);
        let (bx, by) = (three * x0 + (three * x2 - three * x1 - three * x1), three * y0 + (three * y2 - three * y1 - three * y1));
        let (cx, cy) = (three * x1 - three * x0, three * y1 - three * y0);
        let mut prev = p[0];
        for t in (128..=4096).step_by(128) {
            let t: Wrapping<i32> = Wrapping(t);
            let t2 = Wrapping((t * t).0 >> 12);
            let t3 = Wrapping((t * t2).0 >> 12);
            let x = x0.0.wrapping_add((t2 * bx + ax * t3 + cx * t).0 >> 12);
            let y = ((by * t2 + ay * t3 + cy * t).0 >> 12).wrapping_add(y0.0);
            piece(self, prev, (x, y))?;
            prev = (x, y);
        }
        Ok(())
    }

    /// `Ground.method6731`: a rectangle from `x.0` to `x.1` and `y.0` to `y.1` (inclusive),
    /// `width` rows and columns of outline around the fill; `HitbarDefinitions.method3230`
    /// inside the picture, `PlayerModel.method4034` (every edge clamped) otherwise.
    fn rect(&mut self, x: (i32, i32), y: (i32, i32), fill: i32, outline: i32, width: i32) -> Draw {
        let inside = x.0 >= 0 && x.1 <= self.max_x && y.0 >= 0 && y.1 <= self.max_y;
        let (mx, my) = (self.max_x, self.max_y);
        let fx = |v: i32| if inside { v } else { clamp(v, 0, mx) };
        let fy = |v: i32| if inside { v } else { clamp(v, 0, my) };
        let (top, bottom) = (fy(y.0), fy(y.1));
        let (left, right) = (fx(x.0), fx(x.1));
        let (top_end, bottom_start) = (fy(y.0.wrapping_add(width)), fy(y.1.wrapping_sub(width)));
        for row in top..top_end {
            self.span(row, left, right, outline)?;
        }
        for row in (bottom_start + 1..=bottom).rev() {
            self.span(row, left, right, outline)?;
        }
        let (inner_left, inner_right) = (fx(x.0.wrapping_add(width)), fx(x.1.wrapping_sub(width)));
        for row in top_end..=bottom_start {
            self.span(row, left, inner_left, outline)?;
            self.span(row, inner_left, inner_right, fill)?;
            self.span(row, inner_right, right, outline)?;
        }
        Ok(())
    }

    /// `Class299.method5316`: an ellipse with an outline; a circle when both radii match
    /// (`Class532_Sub1.method12838`).
    fn ellipse(&mut self, (x, y): (i32, i32), (rx, ry): (i32, i32), fill: i32, outline: i32, width: i32) -> Draw {
        if rx == ry {
            let inside = x - rx >= 0 && x + rx <= self.max_x && y - rx >= 0 && rx + y <= self.max_y;
            return if inside { self.circle(x, y, rx, fill, outline, width) } else { self.circle_clipped(x, y, rx, fill, outline, width) };
        }
        let inside = x - rx >= 0 && x + rx <= self.max_x && y - ry >= 0 && ry + y <= self.max_y;
        self.oval(x, y, rx, ry, fill, outline, width, !inside)
    }

    /// `Class155.method2637`: a circle with an outline, inside the picture (midpoint circles
    /// for the outer edge and the inner one, `r − width`).
    fn circle(&mut self, x: i32, y: i32, r: i32, fill: i32, outline: i32, width: i32) -> Draw {
        let inner_r = (r - width).max(0);
        self.inner = vec![0; r.max(inner_r + 1).max(0) as usize + 1];
        let (mut i7, mut i9, mut i10, mut i11, mut i12, mut i13, mut i14) = (0, r, -r, inner_r, -inner_r, -1, -1);
        let (a, b) = (x - inner_r, x + inner_r);
        self.span(y, x - r, a, outline)?;
        self.span(y, a, b, fill)?;
        self.span(y, b, x + r, outline)?;
        while i9 > i7 {
            i13 += 2;
            i14 += 2;
            i10 += i13;
            i12 += i14;
            if i12 >= 0 && i11 >= 1 {
                *self.inner.get_mut(i11 as usize).ok_or(Outside)? = i7;
                i11 -= 1;
                i12 -= i11 << 1;
            }
            i7 += 1;
            if i10 >= 0 {
                i9 -= 1;
                i10 -= i9 << 1;
                if i9 >= inner_r {
                    for row in [i9 + y, y - i9] {
                        self.span(row, x - i7, x + i7, outline)?;
                    }
                } else {
                    let i20 = self.inner_at(i9)?;
                    for row in [i9 + y, y - i9] {
                        self.span(row, x - i7, x - i20, outline)?;
                        self.span(row, x - i20, x + i20, fill)?;
                        self.span(row, x + i20, x + i7, outline)?;
                    }
                }
            }
            let (i20, i21) = (x + i9, x - i9);
            if i7 < inner_r {
                let i22 = if i11 < i7 { self.inner_at(i7)? } else { i11 };
                for row in [i7 + y, y - i7] {
                    self.span(row, i21, x - i22, outline)?;
                    self.span(row, x - i22, x + i22, fill)?;
                    self.span(row, x + i22, i20, outline)?;
                }
            } else {
                for row in [i7 + y, y - i7] {
                    self.span(row, i21, i20, outline)?;
                }
            }
        }
        Ok(())
    }

    fn inner_at(&self, i: i32) -> Result<i32, Outside> {
        usize::try_from(i).ok().and_then(|i| self.inner.get(i)).copied().ok_or(Outside)
    }

    /// `Class60.method1174`: the same circle when it crosses the picture's edge (rows outside
    /// skipped, x clamped; the inner x is noted after its step, not before).
    fn circle_clipped(&mut self, x: i32, y: i32, r: i32, fill: i32, outline: i32, width: i32) -> Draw {
        let (mx, my) = (self.max_x, self.max_y);
        let cl = |v: i32| clamp(v, 0, mx);
        let inner_r = (r - width).max(0);
        self.inner = vec![0; r.max(inner_r + 1).max(0) as usize + 1];
        let (mut i7, mut i9, mut i10, mut i11, mut i12, mut i13, mut i14) = (0, r, -r, inner_r, -inner_r, -1, -1);
        if y >= 0 && y <= my {
            let (a, b) = (cl(x - inner_r), cl(x + inner_r));
            self.span(y, cl(x - r), a, outline)?;
            self.span(y, a, b, fill)?;
            self.span(y, b, cl(x + r), outline)?;
        }
        while i9 > i7 {
            i13 += 2;
            i14 += 2;
            i10 += i13;
            i12 += i14;
            if i12 >= 0 && i11 >= 1 {
                i11 -= 1;
                i12 -= i11 << 1;
                *self.inner.get_mut(i11 as usize).ok_or(Outside)? = i7;
            }
            i7 += 1;
            if i10 >= 0 {
                i9 -= 1;
                i10 -= i9 << 1;
                let (up, down) = (y - i9, i9 + y);
                if down >= 0 && up <= my {
                    let rows = [(down, down <= my), (up, up >= 0)];
                    if i9 >= inner_r {
                        let (right, left) = (cl(x + i7), cl(x - i7));
                        for (row, shown) in rows {
                            if shown {
                                self.span(row, left, right, outline)?;
                            }
                        }
                    } else {
                        let i17 = self.inner_at(i9)?;
                        let (right, left, inner_right, inner_left) = (cl(x + i7), cl(x - i7), cl(x + i17), cl(x - i17));
                        for (row, shown) in rows {
                            if shown {
                                self.span(row, left, inner_left, outline)?;
                                self.span(row, inner_left, inner_right, fill)?;
                                self.span(row, inner_right, right, outline)?;
                            }
                        }
                    }
                }
            }
            let (up, down) = (y - i7, i7 + y);
            if down >= 0 && up <= my {
                let (right, left) = (x + i9, x - i9);
                if right >= 0 && left <= mx {
                    let (right, left) = (cl(right), cl(left));
                    let rows = [(down, down <= my), (up, up >= 0)];
                    if i7 < inner_r {
                        let i19 = if i11 < i7 { self.inner_at(i7)? } else { i11 };
                        let (inner_right, inner_left) = (cl(x + i19), cl(x - i19));
                        for (row, shown) in rows {
                            if shown {
                                self.span(row, left, inner_left, outline)?;
                                self.span(row, inner_left, inner_right, fill)?;
                                self.span(row, inner_right, right, outline)?;
                            }
                        }
                    } else {
                        for (row, shown) in rows {
                            if shown {
                                self.span(row, left, right, outline)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// `MaterialProp30.method15241` (inside) and `ClanVarDefinitions.method6824` (clipped): an
    /// ellipse with an outline, the outer and inner (radii − width) edges each stepped by a
    /// midpoint ellipse in Java ints.
    #[allow(clippy::too_many_arguments)]
    fn oval(&mut self, x: i32, y: i32, rx: i32, ry: i32, fill: i32, outline: i32, width: i32, clipped: bool) -> Draw {
        type W = Wrapping<i32>;
        let w = Wrapping;
        let (mx, my) = (self.max_x, self.max_y);
        let cl = |v: i32| if clipped { clamp(v, 0, mx) } else { v };
        let (mut i8, mut i9, mut i10) = (0, ry, 0);
        let (i11, i12) = (rx - width, ry - width);
        let (i13, i14, i15, i16): (W, W, W, W) = (w(rx) * w(rx), w(ry) * w(ry), w(i11) * w(i11), w(i12) * w(i12));
        let (i17, i18, i19, i20) = (i14 << 1, i13 << 1, i16 << 1, i15 << 1);
        let (i21, i22) = (w(ry) << 1, w(i12) << 1);
        let mut i23 = i17 + (w(1) - i21) * i13;
        let mut i24 = i14 - i18 * (i21 - w(1));
        let mut i25 = i19 + (w(1) - i22) * i15;
        let mut i26 = i16 - i20 * (i22 - w(1));
        let (i27, i28, i29, i30) = (i13 << 2, i14 << 2, i15 << 2, i16 << 2);
        let mut i31 = i17 * w(3);
        let mut i32_ = i18 * (i21 - w(3));
        let mut i33 = i19 * w(3);
        let mut i34 = i20 * (i22 - w(3));
        let mut i35 = i28;
        let mut i36 = w(ry - 1) * i27;
        let mut i37 = i30;
        let mut i38 = i29 * w(i12 - 1);
        if !clipped || (y >= 0 && y <= my) {
            let (a, b) = (cl(x - i11), cl(x + i11));
            self.span(y, cl(x - rx), a, outline)?;
            self.span(y, a, b, fill)?;
            self.span(y, b, cl(x + rx), outline)?;
        }
        while i9 > 0 {
            let in_inner = i9 <= i12;
            if in_inner {
                while i25.0 < 0 {
                    i25 += i33;
                    i26 += i37;
                    i33 += i30;
                    i37 += i30;
                    i10 += 1;
                }
                if i26.0 < 0 {
                    i25 += i33;
                    i26 += i37;
                    i33 += i30;
                    i37 += i30;
                    i10 += 1;
                }
                i25 += -i38;
                i26 += -i34;
                i34 -= i29;
                i38 -= i29;
            }
            while i23.0 < 0 {
                i23 += i31;
                i24 += i35;
                i31 += i28;
                i35 += i28;
                i8 += 1;
            }
            if i24.0 < 0 {
                i23 += i31;
                i24 += i35;
                i31 += i28;
                i35 += i28;
                i8 += 1;
            }
            i23 += -i36;
            i24 += -i32_;
            i32_ -= i27;
            i36 -= i27;
            i9 -= 1;
            let (up, down) = (y - i9, i9 + y);
            if clipped && !(down >= 0 && up <= my) {
                continue;
            }
            let (right, left) = (cl(x + i8), cl(x - i8));
            let rows = [(up, !clipped || up >= 0), (down, !clipped || down <= my)];
            for (row, shown) in rows {
                if !shown {
                    continue;
                }
                if in_inner {
                    let (inner_right, inner_left) = (cl(x + i10), cl(x - i10));
                    self.span(row, left, inner_left, outline)?;
                    self.span(row, inner_left, inner_right, fill)?;
                    self.span(row, inner_right, right, outline)?;
                } else {
                    self.span(row, left, right, outline)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rectangle_has_its_outline_around_its_fill() {
        // x and y from 1024 to 3072 of 4096 at 8 × 8: rows and columns 2..=6, width 1. The spans
        // overlap by a pixel, so the fill covers the left outline column and the right one is
        // two wide, as in the reference.
        let rect = Shape::Rect { x: (1024, 3072), y: (1024, 3072), fill: 2, outline: 1, width: 1 };
        let rows = draw(&[rect], 8, 8).unwrap();
        assert_eq!(rows[2], vec![0, 0, 1, 1, 1, 1, 1, 0]);
        assert_eq!(rows[4], vec![0, 0, 1, 2, 2, 1, 1, 0]);
        assert_eq!(rows[6], vec![0, 0, 1, 1, 1, 1, 1, 0]);
        assert!(rows[7].iter().all(|&v| v == 0));
    }

    #[test]
    fn lines_are_clipped_and_circles_are_symmetric() {
        // A line from far left to far right across row 2.
        let line = Shape::Line { from: (-8192, 1024), to: (8192, 1024), colour: 5 };
        let rows = draw(&[line], 8, 8).unwrap();
        assert!(rows[2].iter().all(|&v| v == 5));
        // A circle of radius 2 at (4, 4): row 4 spans 2..=6 (the same overlap), rows 2 and 6 match.
        let circle = Shape::Ellipse { centre: (2048, 2048), radius: (1024, 1024), fill: 2, outline: 1, width: 1 };
        let rows = draw(&[circle], 8, 8).unwrap();
        assert_eq!(rows[4][2..=6], [1, 2, 2, 1, 1]);
        assert_eq!(rows[2], rows[6]);
    }
}
