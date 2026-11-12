//! Synthesized sounds (index 4): up to ten instruments, each oscillators shaped by envelopes,
//! echo and a filter, mixed to 8-bit samples at 22,050 Hz. Port of `SoundEffect` (decode and
//! `mix`), `Instrument` (`decodeInstruments`, `synthesize`, `evaluateWave`), `Envelope` and
//! `Filter`. Integer arithmetic wraps like Java's.

use super::buffer::Reader;
use super::error::Result;
use std::sync::OnceLock;

const RATE: i32 = 22050;

/// `Instrument.noise`: `(new Random(0L).nextInt() & 2) - 1`, 32,768 values.
fn noise() -> &'static [i32] {
    static NOISE: OnceLock<Vec<i32>> = OnceLock::new();
    NOISE.get_or_init(|| {
        // java.util.Random's 48-bit LCG, seeded with 0.
        let mut seed: u64 = 0x5_DEEC_E66D;
        (0..32768)
            .map(|_| {
                seed = (seed.wrapping_mul(0x5_DEEC_E66D).wrapping_add(0xB)) & ((1 << 48) - 1);
                ((seed >> 16) as u32 as i32 & 2) - 1
            })
            .collect()
    })
}

/// `Instrument.sine`.
fn sine() -> &'static [i32] {
    static SINE: OnceLock<Vec<i32>> = OnceLock::new();
    SINE.get_or_init(|| (0..32768).map(|i| ((i as f64 / 5215.1903).sin() * 16384.0) as i32).collect())
}

/// `ByteBuf.readSmart`.
fn smart(r: &mut Reader) -> Result<i32> {
    let peek = r.peek_u8()?;
    Ok(if peek < 128 { r.u8()? as i32 } else { r.u16()? as i32 - 32768 })
}

/// `ByteBuf.readUnsignedSmart` (signed despite the name: -64..).
fn signed_smart(r: &mut Reader) -> Result<i32> {
    let peek = r.peek_u8()?;
    Ok(if peek < 128 { r.u8()? as i32 - 64 } else { r.u16()? as i32 - 49152 })
}

/// `Envelope`.
#[derive(Clone, Debug)]
struct Envelope {
    form: i32,
    start: i32,
    end: i32,
    durations: Vec<i32>,
    peaks: Vec<i32>,
    critical: i32,
    phase: usize,
    step: i32,
    amplitude: i32,
    ticks: i32,
}

impl Envelope {
    fn new() -> Self {
        Envelope {
            form: 0,
            start: 0,
            end: 0,
            durations: vec![0, 65535],
            peaks: vec![0, 65535],
            critical: 0,
            phase: 0,
            step: 0,
            amplitude: 0,
            ticks: 0,
        }
    }

    fn decode(r: &mut Reader) -> Result<Self> {
        let mut e = Envelope::new();
        e.form = r.u8()? as i32;
        e.start = r.i32()?;
        e.end = r.i32()?;
        e.decode_shape(r)?;
        Ok(e)
    }

    fn decode_shape(&mut self, r: &mut Reader) -> Result<()> {
        let n = r.u8()? as usize;
        let (mut durations, mut peaks) = (Vec::with_capacity(n), Vec::with_capacity(n));
        for _ in 0..n {
            durations.push(r.u16()? as i32);
            peaks.push(r.u16()? as i32);
        }
        (self.durations, self.peaks) = (durations, peaks);
        Ok(())
    }

    fn reset(&mut self) {
        (self.critical, self.phase, self.step, self.amplitude, self.ticks) = (0, 0, 0, 0, 0);
    }

    /// `step(period)`.
    fn step(&mut self, period: i32) -> i32 {
        if self.ticks >= self.critical {
            self.amplitude = self.peaks.get(self.phase).copied().unwrap_or(0) << 15;
            self.phase += 1;
            if self.phase >= self.peaks.len() {
                self.phase = self.peaks.len().saturating_sub(1);
            }
            let duration = self.durations.get(self.phase).copied().unwrap_or(0);
            self.critical = (duration as f64 / 65536.0 * period as f64) as i32;
            if self.critical > self.ticks {
                let target = self.peaks.get(self.phase).copied().unwrap_or(0) << 15;
                self.step = target.wrapping_sub(self.amplitude) / (self.critical - self.ticks);
            }
        }
        self.amplitude = self.amplitude.wrapping_add(self.step);
        self.ticks += 1;
        self.amplitude.wrapping_sub(self.step) >> 15
    }
}

/// `Filter`: a pole/zero filter whose coefficients move along an envelope.
#[derive(Clone, Debug, Default)]
struct Filter {
    pairs: [usize; 2],
    phase: [[[i32; 4]; 2]; 2],
    magnitude: [[[i32; 4]; 2]; 2],
    unity: [i32; 2],
    /// `coefficient_float`, `coefficient_int`, `invUnity_*` (static in the reference).
    coef_f: [[f32; 8]; 2],
    coef_i: [[i32; 8]; 2],
    inv_unity_f: f32,
    inv_unity_i: i32,
}

impl Filter {
    fn decode(r: &mut Reader, envelope: &mut Envelope) -> Result<Self> {
        let mut f = Filter::default();
        let count = r.u8()? as usize;
        f.pairs = [count >> 4, count & 0xf];
        if count != 0 {
            f.unity = [r.u16()? as i32, r.u16()? as i32];
            let migrated = r.u8()? as i32;
            for dir in 0..2 {
                for term in 0..f.pairs[dir] {
                    f.phase[dir][0][term] = r.u16()? as i32;
                    f.magnitude[dir][0][term] = r.u16()? as i32;
                }
            }
            for dir in 0..2 {
                for term in 0..f.pairs[dir] {
                    if migrated & (1 << (dir * 4) << term) != 0 {
                        f.phase[dir][1][term] = r.u16()? as i32;
                        f.magnitude[dir][1][term] = r.u16()? as i32;
                    } else {
                        f.phase[dir][1][term] = f.phase[dir][0][term];
                        f.magnitude[dir][1][term] = f.magnitude[dir][0][term];
                    }
                }
            }
            if migrated != 0 || f.unity[1] != f.unity[0] {
                envelope.decode_shape(r)?;
            }
        }
        Ok(f)
    }

    fn normalize(f: f32) -> f32 {
        let v = 32.703197f32 * 2f64.powf(f as f64) as f32;
        // `3.1415927F` is exactly `f32::consts::PI`.
        v * std::f32::consts::PI / 11025.0
    }

    fn adapt_magnitude(&self, dir: usize, k: usize, t: f32) -> f32 {
        let m = &self.magnitude[dir];
        let mut alpha = m[0][k] as f32 + t * (m[1][k] - m[0][k]) as f32;
        alpha *= 0.0015258789;
        1.0 - 10f64.powf((-alpha / 20.0) as f64) as f32
    }

    fn adapt_phase(&self, dir: usize, k: usize, t: f32) -> f32 {
        let p = &self.phase[dir];
        let mut v = p[0][k] as f32 + t * (p[1][k] - p[0][k]) as f32;
        v *= 1.2207031E-4;
        Filter::normalize(v)
    }

    /// `compute(dir, t)`: the number of coefficients for one direction.
    fn compute(&mut self, dir: usize, t: f32) -> usize {
        if dir == 0 {
            let mut a0 = self.unity[0] as f32 + (self.unity[1] - self.unity[0]) as f32 * t;
            a0 *= 0.0030517578;
            self.inv_unity_f = 0.1f64.powf((a0 / 20.0) as f64) as f32;
            self.inv_unity_i = (self.inv_unity_f * 65536.0) as i32;
        }
        let n = self.pairs[dir];
        if n == 0 {
            return 0;
        }
        let m = self.adapt_magnitude(dir, 0, t);
        let ph = self.adapt_phase(dir, 0, t);
        let c = &mut self.coef_f;
        c[dir][0] = -2.0 * m * (ph as f64).cos() as f32;
        c[dir][1] = m * m;
        for k in 1..n {
            let m = self.adapt_magnitude(dir, k, t);
            let ph = self.adapt_phase(dir, k, t);
            let c = &mut self.coef_f;
            let f5 = -2.0 * m * (ph as f64).cos() as f32;
            let f6 = m * m;
            c[dir][k * 2 + 1] = c[dir][k * 2 - 1] * f6;
            c[dir][k * 2] = c[dir][k * 2 - 1] * f5 + c[dir][k * 2 - 2] * f6;
            for i in (2..=k * 2 - 1).rev() {
                c[dir][i] += c[dir][i - 1] * f5 + c[dir][i - 2] * f6;
            }
            c[dir][1] += c[dir][0] * f5 + f6;
            c[dir][0] += f5;
        }
        if dir == 0 {
            for i in 0..n * 2 {
                self.coef_f[0][i] *= self.inv_unity_f;
            }
        }
        for i in 0..n * 2 {
            self.coef_i[dir][i] = (self.coef_f[dir][i] * 65536.0) as i32;
        }
        n * 2
    }
}

/// `Instrument`.
#[derive(Clone, Debug)]
struct Instrument {
    pitch: Envelope,
    volume: Envelope,
    pitch_mod: Option<(Envelope, Envelope)>,
    volume_mod: Option<(Envelope, Envelope)>,
    gate: Option<(Envelope, Envelope)>,
    oscillators: Vec<(i32, i32, i32)>,
    delay_time: i32,
    delay_decay: i32,
    duration: i32,
    offset: i32,
    filter: Filter,
    filter_envelope: Envelope,
}

/// `evaluateWave(phase, amplitude, form)`: 1 square, 2 sine, 3 saw, 4 noise.
fn wave(phase: i32, amplitude: i32, form: i32) -> i32 {
    match form {
        1 => {
            if phase & 0x7fff < 16384 {
                amplitude
            } else {
                -amplitude
            }
        }
        2 => sine()[(phase & 0x7fff) as usize].wrapping_mul(amplitude) >> 14,
        3 => (amplitude.wrapping_mul(phase & 0x7fff) >> 14) - amplitude,
        4 => amplitude.wrapping_mul(noise()[((phase / 2607) & 0x7fff) as usize]),
        _ => 0,
    }
}

impl Instrument {
    fn decode(r: &mut Reader) -> Result<Self> {
        let pitch = Envelope::decode(r)?;
        let volume = Envelope::decode(r)?;
        let pair = |r: &mut Reader| -> Result<Option<(Envelope, Envelope)>> {
            if r.peek_u8()? != 0 {
                Ok(Some((Envelope::decode(r)?, Envelope::decode(r)?)))
            } else {
                r.u8()?;
                Ok(None)
            }
        };
        let pitch_mod = pair(r)?;
        let volume_mod = pair(r)?;
        let gate = pair(r)?;
        let mut oscillators = Vec::new();
        for _ in 0..10 {
            let volume = smart(r)?;
            if volume == 0 {
                break;
            }
            if oscillators.len() == 5 {
                // The reference keeps five and would throw on a sixth.
                return super::error::corrupt("sound with more than five oscillators");
            }
            oscillators.push((volume, signed_smart(r)?, smart(r)?));
        }
        let delay_time = smart(r)?;
        let delay_decay = smart(r)?;
        let duration = r.u16()? as i32;
        let offset = r.u16()? as i32;
        let mut filter_envelope = Envelope::new();
        let filter = Filter::decode(r, &mut filter_envelope)?;
        Ok(Instrument {
            pitch,
            volume,
            pitch_mod,
            volume_mod,
            gate,
            oscillators,
            delay_time,
            delay_decay,
            duration,
            offset,
            filter,
            filter_envelope,
        })
    }

    /// `synthesize(mixDuration, instrDuration)`.
    fn synthesize(&mut self, len: usize, millis: i32) -> Vec<i32> {
        let mut out = vec![0i32; len];
        if millis < 10 {
            return out;
        }
        let n = len as i32;
        let fs = len as f64 / millis as f64;
        self.pitch.reset();
        self.volume.reset();
        let base = |e: &Envelope| ((e.end.wrapping_sub(e.start)) as f64 * 32.768 / fs) as i32;
        let (mut pm_step, mut pm_base, mut pm_phase) = (0, 0, 0i32);
        if let Some((m, a)) = &mut self.pitch_mod {
            m.reset();
            a.reset();
            pm_step = base(m);
            pm_base = (m.start as f64 * 32.768 / fs) as i32;
        }
        let (mut vm_step, mut vm_base, mut vm_phase) = (0, 0, 0i32);
        if let Some((m, a)) = &mut self.volume_mod {
            m.reset();
            a.reset();
            vm_step = base(m);
            vm_base = (m.start as f64 * 32.768 / fs) as i32;
        }
        let oscs: Vec<(i32, i32, i32, i32)> = self
            .oscillators
            .iter()
            .map(|&(volume, pitch, delay)| {
                (
                    (delay as f64 * fs) as i32,
                    (volume << 14) / 100,
                    ((self.pitch.end.wrapping_sub(self.pitch.start)) as f64 * 32.768 * 1.0057929410678534f64.powi(pitch) / fs) as i32,
                    (self.pitch.start as f64 * 32.768 / fs) as i32,
                )
            })
            .collect();
        let mut phases = vec![0i32; oscs.len()];
        for i in 0..n {
            let mut pitch_change = self.pitch.step(n);
            let mut volume_change = self.volume.step(n);
            if let Some((m, a)) = &mut self.pitch_mod {
                let md = m.step(n);
                let amp = a.step(n);
                pitch_change = pitch_change.wrapping_add(wave(pm_phase, amp, m.form) >> 1);
                pm_phase = pm_phase.wrapping_add(pm_base.wrapping_add(md.wrapping_mul(pm_step) >> 16));
            }
            if let Some((m, a)) = &mut self.volume_mod {
                let md = m.step(n);
                let amp = a.step(n);
                volume_change = volume_change.wrapping_mul((wave(vm_phase, amp, m.form) >> 1) + 32768) >> 15;
                vm_phase = vm_phase.wrapping_add(vm_base.wrapping_add(md.wrapping_mul(vm_step) >> 16));
            }
            for (k, &(delay, vol, step, base_step)) in oscs.iter().enumerate() {
                let at = delay + i;
                if at < n {
                    out[at as usize] =
                        out[at as usize].wrapping_add(wave(phases[k], volume_change.wrapping_mul(vol) >> 15, self.pitch.form));
                    phases[k] = phases[k].wrapping_add((pitch_change.wrapping_mul(step) >> 16).wrapping_add(base_step));
                }
            }
        }
        if let Some((on, off)) = &mut self.gate {
            on.reset();
            off.reset();
            let (mut counter, mut muted) = (0, true);
            for v in out.iter_mut() {
                let on_step = on.step(n);
                let off_step = off.step(n);
                let threshold = if muted { on_step } else { off_step }.wrapping_mul(on.end.wrapping_sub(on.start)) >> 8;
                let threshold = threshold.wrapping_add(on.start);
                counter += 256;
                if counter >= threshold {
                    counter = 0;
                    muted = !muted;
                }
                if muted {
                    *v = 0;
                }
            }
        }
        if self.delay_time > 0 && self.delay_decay > 0 {
            let delay = (self.delay_time as f64 * fs) as usize;
            for i in delay..len {
                out[i] = out[i].wrapping_add(out[i - delay].wrapping_mul(self.delay_decay) / 100);
            }
        }
        if self.filter.pairs[0] > 0 || self.filter.pairs[1] > 0 {
            self.apply_filter(&mut out);
        }
        for v in &mut out {
            *v = (*v).clamp(-32768, 32767);
        }
        out
    }

    /// The filter part of `synthesize`, in its three stretches.
    fn apply_filter(&mut self, out: &mut [i32]) {
        let len = out.len();
        let n_i = len as i32;
        let mul = |v: i32, c: i32| ((v as i64 * c as i64) >> 16) as i32;
        self.filter_envelope.reset();
        let mut t = self.filter_envelope.step(n_i + 1);
        let mut m = self.filter.compute(0, t as f32 / 65536.0);
        let mut nn = self.filter.compute(1, t as f32 / 65536.0);
        if len < m + nn {
            return;
        }
        let mut i = 0usize;
        let first = nn.min(len - m);
        while i < first {
            let f = &self.filter;
            let mut y = mul(out[i + m], f.inv_unity_i);
            for k in 0..m {
                y = y.wrapping_add(mul(out[i + m - 1 - k], f.coef_i[0][k]));
            }
            for k in 0..i {
                y = y.wrapping_sub(mul(out[i - 1 - k], f.coef_i[1][k]));
            }
            out[i] = y;
            t = self.filter_envelope.step(n_i + 1);
            i += 1;
        }
        let mut limit = 128usize;
        loop {
            limit = limit.min(len - m);
            while i < limit {
                let f = &self.filter;
                let mut y = mul(out[i + m], f.inv_unity_i);
                for k in 0..m {
                    y = y.wrapping_add(mul(out[i + m - 1 - k], f.coef_i[0][k]));
                }
                for k in 0..nn {
                    y = y.wrapping_sub(mul(out[i - 1 - k], f.coef_i[1][k]));
                }
                out[i] = y;
                t = self.filter_envelope.step(n_i + 1);
                i += 1;
            }
            if i >= len - m {
                while i < len {
                    let f = &self.filter;
                    let mut y = 0i32;
                    for k in (i + m - len)..m {
                        y = y.wrapping_add(mul(out[i + m - 1 - k], f.coef_i[0][k]));
                    }
                    for k in 0..nn {
                        y = y.wrapping_sub(mul(out[i - 1 - k], f.coef_i[1][k]));
                    }
                    out[i] = y;
                    self.filter_envelope.step(n_i + 1);
                    i += 1;
                }
                return;
            }
            m = self.filter.compute(0, t as f32 / 65536.0);
            nn = self.filter.compute(1, t as f32 / 65536.0);
            limit += 128;
        }
    }
}

/// A mixed synthesized sound (`Node_Sub26_Sub1_Sub2`): signed 8-bit samples at 22,050 Hz.
pub struct SynthSample {
    pub pcm: Vec<i8>,
    pub loop_start: i32,
    pub loop_end: i32,
}

/// `SoundEffect`: decode, then `getMixedAudio`.
pub fn synthesize(data: &[u8]) -> Result<SynthSample> {
    let mut r = Reader::new(data);
    let mut instruments = Vec::new();
    for _ in 0..10 {
        if r.peek_u8()? != 0 {
            instruments.push(Instrument::decode(&mut r)?);
        } else {
            r.u8()?;
        }
    }
    let loop_begin = r.u16()? as i32;
    let loop_end = r.u16()? as i32;
    let duration = instruments.iter().map(|i| i.duration + i.offset).max().unwrap_or(0);
    let mut mixed = vec![0i8; (duration * RATE / 1000).max(0) as usize];
    for inst in &mut instruments {
        let len = (inst.duration * RATE / 1000) as usize;
        let offset = (inst.offset * RATE / 1000) as usize;
        let samples = inst.synthesize(len, inst.duration);
        for (j, s) in samples.iter().enumerate() {
            let Some(slot) = mixed.get_mut(j + offset) else { break };
            let mut v = (s >> 8) + *slot as i32;
            if (v + 128) & -256 != 0 {
                v = v >> 31 ^ 0x7f;
            }
            *slot = v as i8;
        }
    }
    Ok(SynthSample { pcm: mixed, loop_start: loop_begin * RATE / 1000, loop_end: loop_end * RATE / 1000 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_follows_java_random_zero() {
        // new Random(0).nextInt() is -1155484576; (-1155484576 & 2) - 1 = -1.
        assert_eq!(noise()[0], -1);
        assert!(noise().iter().all(|&v| v == 1 || v == -1));
    }

    #[test]
    fn waves_match_the_reference() {
        assert_eq!(wave(0, 100, 1), 100);
        assert_eq!(wave(20000, 100, 1), -100);
        assert_eq!(wave(16384, 100, 3), 0);
        assert_eq!(wave(5, 7, 0), 0);
    }

    #[test]
    fn silent_sound_mixes_to_nothing() {
        // Ten absent instruments, then loop 0..0.
        let data = [0u8; 14];
        assert!(synthesize(&data).unwrap().pcm.is_empty());
    }
}
