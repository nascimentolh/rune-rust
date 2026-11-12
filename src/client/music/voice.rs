//! One playing sample: position in 8.8 fixed point, rate (negative = backwards), volume and pan
//! with linear ramps, loops. Port of `Node_Sub15_Sub5` and its mixers `Node_Sub15_Sub5_Sub1`
//! (float samples) / `Node_Sub15_Sub5_Sub2` (8-bit samples).
//!
//! ponytail: the reference has a dozen unrolled copies of the mixing loop (mono/stereo, rate 256
//! or not, ramp or not); here one loop per direction does all of them. 8-bit samples are mixed as
//! floats (b / 128), which can round the last bit differently.

use std::rc::Rc;

/// A sample as voices play it (`Node_Sub26_Sub1`).
pub struct SampleData {
    /// `anInt9750`: the sample's own rate.
    pub rate: i32,
    pub pcm: Vec<f32>,
    /// `anInt9749` / `anInt9751`.
    pub loop_start: i32,
    pub loop_end: i32,
    /// `aBool9752`: loop back and forth.
    pub ping_pong: bool,
}

impl SampleData {
    fn at(&self, i: i32) -> f32 {
        self.pcm.get(i as usize).copied().unwrap_or(0.0)
    }
}

/// `HostNameIdentifier.method488`: left gain.
fn left(volume: i32, pan: i32) -> i32 {
    if pan < 0 {
        volume
    } else {
        (volume as f64 * (1.220703125E-4 * (16384 - pan) as f64).sqrt() + 0.5) as i32
    }
}

/// `MaterialPropTexture.method15390`: right gain.
fn right(volume: i32, pan: i32) -> i32 {
    if pan < 0 {
        -volume
    } else {
        (volume as f64 * (1.220703125E-4 * pan as f64).sqrt() + 0.5) as i32
    }
}

pub struct Voice {
    sample: Rc<SampleData>,
    /// `anInt9853`, `anInt9855`.
    pub pos: i32,
    rate: i32,
    /// `anInt9854`: -1 loops forever, n loops n times, 0 plays through.
    pub loops: i32,
    /// `anInt9848` (mono volume), `anInt9845` / `anInt9850` (left / right).
    volume: i32,
    left: i32,
    right: i32,
    /// `anInt9846` / `anInt9847`: targets (`i32::MIN` volume = fading out to stop).
    target_volume: i32,
    target_pan: i32,
    /// `anInt9858`: samples of ramp left; `anInt9849/9852/9851`: per-sample steps.
    ramp: i32,
    step_volume: i32,
    step_left: i32,
    step_right: i32,
    /// Unlinked: done playing.
    pub done: bool,
}

impl Voice {
    /// `method15225` → the `Node_Sub15_Sub5_Sub1/Sub2` constructor.
    pub fn new(sample: Rc<SampleData>, rate: i32, volume: i32, pan: i32) -> Self {
        let mut v = Voice {
            sample,
            pos: 0,
            rate,
            loops: 0,
            volume: 0,
            left: 0,
            right: 0,
            target_volume: volume,
            target_pan: pan,
            ramp: 0,
            step_volume: 0,
            step_left: 0,
            step_right: 0,
            done: false,
        };
        v.settle();
        v
    }

    pub fn sample(&self) -> &Rc<SampleData> {
        &self.sample
    }

    /// `method15332`: current = targets.
    fn settle(&mut self) {
        self.volume = self.target_volume;
        self.left = left(self.target_volume, self.target_pan);
        self.right = right(self.target_volume, self.target_pan);
    }

    /// `method15308`: a new rate keeping the direction.
    pub fn set_rate(&mut self, rate: i32) {
        self.rate = if self.rate < 0 { -rate } else { rate };
    }

    /// `method15313`: play backwards.
    pub fn reverse(&mut self) {
        self.rate = -self.rate.abs();
    }

    /// `method15316`.
    pub fn target_pan(&self) -> i32 {
        self.target_pan.max(-1)
    }

    /// `method15317`.
    pub fn set_position(&mut self, pos: i32) {
        let end = (self.sample.pcm.len() as i32) << 8;
        self.pos = pos.clamp(-1, end);
    }

    /// `method15319`: outside the sample.
    pub fn outside(&self) -> bool {
        self.pos < 0 || self.pos >= (self.sample.pcm.len() as i32) << 8
    }

    /// `method15311`: still ramping.
    pub fn ramping(&self) -> bool {
        self.ramp != 0
    }

    /// `method15364`.
    fn set_now(&mut self, volume: i32, pan: i32) {
        self.target_volume = volume;
        self.target_pan = pan;
        self.ramp = 0;
        self.settle();
    }

    /// `method15321`: ramp to (volume, pan) over at most `n` samples.
    pub fn ramp_to(&mut self, n: i32, volume: i32, pan: i32) {
        if n == 0 {
            return self.set_now(volume, pan);
        }
        let (l, r) = (left(volume, pan), right(volume, pan));
        if l == self.left && r == self.right {
            self.ramp = 0;
            return;
        }
        let biggest = [volume - self.volume, self.volume - volume, l - self.left, self.left - l, r - self.right, self.right - r]
            .into_iter()
            .max()
            .unwrap_or(0);
        let n = n.min(biggest);
        self.ramp = n;
        self.target_volume = volume;
        self.target_pan = pan;
        self.step_volume = (volume - self.volume) / n;
        self.step_left = (l - self.left) / n;
        self.step_right = (r - self.right) / n;
    }

    /// `method15322`: fade out over at most `n` samples, then stop.
    pub fn fade_out(&mut self, n: i32) {
        if n == 0 {
            self.set_now(0, self.target_pan());
            self.done = true;
        } else if self.left == 0 && self.right == 0 {
            (self.ramp, self.target_volume, self.volume) = (0, 0, 0);
            self.done = true;
        } else {
            let biggest = [-self.volume, self.volume, -self.left, self.left, -self.right, self.right].into_iter().max().unwrap_or(0);
            let n = n.min(biggest);
            self.ramp = n;
            self.target_volume = i32::MIN;
            self.step_volume = -self.volume / n;
            self.step_left = -self.left / n;
            self.step_right = -self.right / n;
        }
    }

    /// `method15354`.
    fn stop_ramp(&mut self) {
        if self.ramp != 0 {
            if self.target_volume == i32::MIN {
                self.target_volume = 0;
            }
            self.ramp = 0;
            self.settle();
        }
    }

    /// `method15324`: after a ramp, step by ±1 toward any rounding left; true when a fade-out ended.
    fn next_ramp(&mut self) -> bool {
        let (v, l, r) = if self.target_volume == i32::MIN {
            (0, 0, 0)
        } else {
            (self.target_volume, left(self.target_volume, self.target_pan), right(self.target_volume, self.target_pan))
        };
        if v == self.volume && l == self.left && r == self.right {
            if self.target_volume == i32::MIN {
                (self.target_volume, self.right, self.left, self.volume) = (0, 0, 0, 0);
                self.done = true;
                return true;
            }
            self.settle();
            return false;
        }
        self.step_volume = (v - self.volume).signum();
        self.ramp = (v - self.volume).abs();
        for (cur, to, step) in [(self.left, l, &mut self.step_left), (self.right, r, &mut self.step_right)] {
            *step = (to - cur).signum();
            let d = (to - cur).abs();
            if d != 0 && (self.ramp == 0 || self.ramp > d) {
                self.ramp = d;
            }
        }
        false
    }

    /// Adds `n` samples to `out` (interleaved when `stereo`) from frame `at`
    /// (`Node_Sub15_Sub5_Sub1.method12230`).
    pub fn mix(&mut self, out: &mut [i32], at: usize, n: usize, stereo: bool) {
        if self.done {
            return;
        }
        let mut scratch;
        let (out, at) = if self.target_volume == 0 && self.ramp == 0 {
            // `method12231`: silent, so only time passes.
            scratch = vec![0i32; n * if stereo { 2 } else { 1 }];
            (&mut scratch[..], 0usize)
        } else {
            (out, at)
        };
        let s = Rc::clone(&self.sample);
        let ls = s.loop_start << 8;
        let le = s.loop_end << 8;
        let len = (s.pcm.len() as i32) << 8;
        let span = le - ls;
        if span <= 0 {
            self.loops = 0;
        }
        let end = (at + n) as i32;
        let mut o = at as i32;
        if self.pos < 0 {
            if self.rate <= 0 {
                self.stop_ramp();
                self.done = true;
                return;
            }
            self.pos = 0;
        }
        if self.pos >= len {
            if self.rate >= 0 {
                self.stop_ramp();
                self.done = true;
                return;
            }
            self.pos = len - 1;
        }
        let first = s.at(s.loop_start);
        let last = s.at(s.loop_end - 1);
        if self.loops < 0 {
            if s.ping_pong {
                if self.rate < 0 {
                    o = self.backward(out, o, ls, end, first, stereo);
                    if self.pos >= ls {
                        return;
                    }
                    self.pos = ls + ls - 1 - self.pos;
                    self.rate = -self.rate;
                }
                loop {
                    o = self.forward(out, o, le, end, last, stereo);
                    if self.pos < le {
                        break;
                    }
                    self.pos = le + le - 1 - self.pos;
                    self.rate = -self.rate;
                    o = self.backward(out, o, ls, end, first, stereo);
                    if self.pos >= ls {
                        break;
                    }
                    self.pos = ls + ls - 1 - self.pos;
                    self.rate = -self.rate;
                }
            } else if self.rate < 0 {
                loop {
                    o = self.backward(out, o, ls, end, last, stereo);
                    if self.pos >= ls {
                        break;
                    }
                    self.pos = le - 1 - (le - 1 - self.pos) % span;
                }
            } else {
                loop {
                    o = self.forward(out, o, le, end, first, stereo);
                    if self.pos < le {
                        break;
                    }
                    self.pos = ls + (self.pos - ls) % span;
                }
            }
            return;
        }
        if self.loops > 0 {
            if s.ping_pong {
                let mut finished_loops = false;
                if self.rate < 0 {
                    o = self.backward(out, at as i32, ls, end, first, stereo);
                    if self.pos >= ls {
                        return;
                    }
                    self.pos = ls + ls - 1 - self.pos;
                    self.rate = -self.rate;
                    self.loops -= 1;
                    finished_loops = self.loops == 0;
                }
                while !finished_loops {
                    o = self.forward(out, o, le, end, last, stereo);
                    if self.pos < le {
                        return;
                    }
                    self.pos = le + le - 1 - self.pos;
                    self.rate = -self.rate;
                    self.loops -= 1;
                    if self.loops == 0 {
                        break;
                    }
                    o = self.backward(out, o, ls, end, first, stereo);
                    if self.pos >= ls {
                        return;
                    }
                    self.pos = ls + ls - 1 - self.pos;
                    self.rate = -self.rate;
                    self.loops -= 1;
                    finished_loops = self.loops == 0;
                }
            } else if self.rate < 0 {
                loop {
                    o = self.backward(out, o, ls, end, last, stereo);
                    if self.pos >= ls {
                        return;
                    }
                    let times = (le - 1 - self.pos) / span;
                    if times >= self.loops {
                        self.pos += span * self.loops;
                        self.loops = 0;
                        break;
                    }
                    self.pos += span * times;
                    self.loops -= times;
                }
            } else {
                loop {
                    o = self.forward(out, o, le, end, first, stereo);
                    if self.pos < le {
                        return;
                    }
                    let times = (self.pos - ls) / span;
                    if times >= self.loops {
                        self.pos -= span * self.loops;
                        self.loops = 0;
                        break;
                    }
                    self.pos -= span * times;
                    self.loops -= times;
                }
            }
        }
        if self.rate < 0 {
            self.backward(out, o, 0, end, 0.0, stereo);
            if self.pos < 0 {
                self.pos = -1;
                self.stop_ramp();
                self.done = true;
            }
        } else {
            self.forward(out, o, len, end, 0.0, stereo);
            if self.pos >= len {
                self.pos = len;
                self.stop_ramp();
                self.done = true;
            }
        }
    }

    /// `method15769`: forwards until position `limit` or frame `end`, in ramp chunks.
    fn forward(&mut self, out: &mut [i32], o: i32, limit: i32, end: i32, after: f32, stereo: bool) -> i32 {
        self.walk(out, o, limit, end, after, stereo, true)
    }

    /// `method15732`: backwards until position `limit` or frame `end`.
    fn backward(&mut self, out: &mut [i32], o: i32, limit: i32, end: i32, before: f32, stereo: bool) -> i32 {
        self.walk(out, o, limit, end, before, stereo, false)
    }

    #[allow(clippy::too_many_arguments)] // mirrors the reference's segment call: buffer, frames, limit, edge sample
    fn walk(&mut self, out: &mut [i32], mut o: i32, limit: i32, end: i32, edge: f32, stereo: bool, forwards: bool) -> i32 {
        loop {
            if self.ramp <= 0 {
                return self.segment(out, o, limit, end, edge, stereo, forwards, false);
            }
            let chunk_end = (o + self.ramp).min(end);
            self.ramp += o;
            o = self.segment(out, o, limit, chunk_end, edge, stereo, forwards, true);
            self.ramp -= o;
            if self.ramp != 0 {
                return o;
            }
            if self.next_ramp() {
                return end;
            }
        }
    }

    /// One mixing loop (`method15746`/`15741`/`15747`/`15735` forwards, `15729`/… backwards):
    /// linear interpolation `s·256 + (next − s)·frac`, × (volume × 128) `>> 6`.
    #[allow(clippy::too_many_arguments)]
    fn segment(
        &mut self,
        out: &mut [i32],
        mut o: i32,
        limit: i32,
        end: i32,
        edge: f32,
        stereo: bool,
        forwards: bool,
        ramping: bool,
    ) -> i32 {
        let s = Rc::clone(&self.sample);
        let r = self.rate;
        let (mut v, mut l, mut rt) = (self.volume * 128, self.left * 128, self.right * 128);
        let (sv, sl, sr) = (self.step_volume * 128, self.step_left * 128, self.step_right * 128);
        let mut pos = self.pos;
        let mut put = |o: i32, f: f32, v: &mut i32, l: &mut i32, rt: &mut i32| {
            if stereo {
                let i = o as usize * 2;
                if let Some(slot) = out.get_mut(i) {
                    *slot = slot.wrapping_add((f * *l as f32) as i32 >> 6);
                }
                if let Some(slot) = out.get_mut(i + 1) {
                    *slot = slot.wrapping_add((f * *rt as f32) as i32 >> 6);
                }
            } else if let Some(slot) = out.get_mut(o as usize) {
                *slot = slot.wrapping_add((f * *v as f32) as i32 >> 6);
            }
            if ramping {
                *v += sv;
                *l += sl;
                *rt += sr;
            }
        };
        let cap = |x: i32| if r == 0 { end } else { x.min(end) };
        if forwards {
            let n1 = cap(o + if r == 0 { 0 } else { (limit - pos + r - 257) / r });
            while o < n1 {
                let i = pos >> 8;
                let a = s.at(i);
                let f = a * 256.0 + (s.at(i + 1) - a) * (pos & 0xff) as f32;
                put(o, f, &mut v, &mut l, &mut rt);
                o += 1;
                pos += r;
            }
            let n2 = cap(o + if r == 0 { 0 } else { (limit - pos + r - 1) / r });
            while o < n2 {
                let a = s.at(pos >> 8);
                let f = a * 256.0 + (edge - a) * (pos & 0xff) as f32;
                put(o, f, &mut v, &mut l, &mut rt);
                o += 1;
                pos += r;
            }
        } else {
            let n1 = cap(o + if r == 0 { 0 } else { (limit + 256 - pos + r) / r });
            while o < n1 {
                let i = pos >> 8;
                let a = s.at(i - 1);
                let f = a * 256.0 + (s.at(i) - a) * (pos & 0xff) as f32;
                put(o, f, &mut v, &mut l, &mut rt);
                o += 1;
                pos += r;
            }
            let n2 = cap(o + if r == 0 { 0 } else { (limit - pos + r) / r });
            while o < n2 {
                let f = edge * 256.0 + (s.at(pos >> 8) - edge) * (pos & 0xff) as f32;
                put(o, f, &mut v, &mut l, &mut rt);
                o += 1;
                pos += r;
            }
        }
        if ramping {
            self.volume = v / 128;
            self.left = l / 128;
            self.right = rt / 128;
        }
        self.pos = pos;
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_law_is_constant_power() {
        assert_eq!((left(1000, 8192), right(1000, 8192)), (1000, 1000));
        assert_eq!((left(1000, 0), right(1000, 0)), (1414, 0));
    }

    #[test]
    fn plays_a_sample_at_its_rate_then_stops() {
        let s = Rc::new(SampleData { rate: 22050, pcm: vec![0.5; 4], loop_start: 0, loop_end: 0, ping_pong: false });
        let mut v = Voice::new(s, 256, 64, -1);
        let mut out = vec![0i32; 8];
        v.mix(&mut out, 0, 8, false);
        assert!(out[0] != 0 && v.done);
        assert_eq!(out[6], 0);
    }
}
