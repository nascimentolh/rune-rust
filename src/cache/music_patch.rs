//! Music patches (index 15, one per MIDI program): which sample each of the 128 notes plays,
//! its pitch, volume, pan and envelope. Port of `Node_Sub14`'s constructor, with `Class110`
//! (an envelope) and `Class371.method6349` (division rounding toward negative infinity).
//!
//! Byte fields keep Java's signed `byte` arithmetic (`as i8`) so values wrap exactly as there.

use super::buffer::Reader;
use super::error::{corrupt, Result};

/// `Class110`: attack/release envelopes and vibrato of a group of notes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Envelope {
    /// `aByteArray1097`: release points (time, level) pairs.
    pub release: Option<Vec<i8>>,
    /// `aByteArray1094`: attack points, starting (0, 64).
    pub attack: Option<Vec<i8>>,
    /// `anInt1095`, `anInt1098`, `anInt1096`, `anInt1099`, `anInt1101`, `anInt1100`, `anInt1102`.
    pub release_rate: i32,
    pub attack_rate: i32,
    pub decay: i32,
    pub sustain: i32,
    pub vibrato_depth: i32,
    pub vibrato_rate: i32,
    pub vibrato_delay: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Patch {
    /// `anIntArray7596`: per note, 0 = silent, else (sample id << 2 | kind) + 1, where kind's bit 0
    /// picks a Vorbis instrument (index 14) over a synthesized sound (index 4) and bit 1 loops.
    pub samples: [i32; 128],
    /// `aShortArray7590`: per note, the sample's pitch (with the loop bit at 0x8000).
    pub pitch: [i16; 128],
    /// `aByteArray7592`: per note volume.
    pub volume: [i8; 128],
    /// `aByteArray7591`: per note pan.
    pub pan: [i8; 128],
    /// `aByteArray7595`: per note exclusive class.
    pub exclusive: [i8; 128],
    /// `aClass110Array7594`: per note, an index into `envelopes`.
    pub envelope: [Option<usize>; 128],
    pub envelopes: Vec<Envelope>,
    /// `anInt7593`: the patch's own volume.
    pub base_volume: i32,
}

/// `Class371.method6349`.
fn div_floor(a: i32, b: i32) -> i32 {
    let neg = (a as u32 >> 31) as i32;
    (a + neg) / b - neg
}

/// A zero-terminated run of bytes (run lengths), the terminator skipped.
fn runs(r: &mut Reader) -> Result<Vec<i8>> {
    let mut out = Vec::new();
    loop {
        match r.u8()? {
            0 => return Ok(out),
            b => out.push(b as i8),
        }
    }
}

/// The next run length from `lengths` (-1 once they run out, without moving on), as the
/// patch's loops read them.
fn next_len(lengths: &[i8], i: &mut usize) -> i32 {
    match lengths.get(*i) {
        Some(&b) => {
            *i += 1;
            b as i32
        }
        None => -1,
    }
}

impl Patch {
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return corrupt("empty music patch");
        }
        let mut r = Reader::new(data);
        // Three run lists, the first two each followed by as many value bytes (+1).
        let exclusive_runs = runs(&mut r)?;
        let exclusive_values = r.position();
        r.bytes(exclusive_runs.len() + 1)?;
        let pan_runs = runs(&mut r)?;
        let pan_values = r.position();
        r.bytes(pan_runs.len() + 1)?;
        let envelope_runs = runs(&mut r)?;
        let count = envelope_runs.len() + 1;
        let mut envelope_of_run = vec![0usize; count];
        let groups = if count > 1 {
            envelope_of_run[1] = 1;
            let (mut current, mut next) = (1usize, 2usize);
            for slot in envelope_of_run.iter_mut().skip(2) {
                let v = r.u8()? as usize;
                if v == 0 {
                    current = next;
                    next += 1;
                } else {
                    current = if v <= current { v - 1 } else { v };
                }
                *slot = current;
            }
            next
        } else {
            count
        };
        let mut envelopes = vec![Envelope::default(); groups];
        for e in &mut envelopes {
            let n = r.u8()? as usize;
            if n > 0 {
                e.release = Some(vec![0; n * 2]);
            }
            let n = r.u8()? as usize;
            if n > 0 {
                let mut attack = vec![0i8; n * 2 + 2];
                attack[1] = 64;
                e.attack = Some(attack);
            }
        }
        let n = r.u8()? as usize;
        let mut volume_curve = (n > 0).then(|| vec![0i8; n * 2]);
        let n = r.u8()? as usize;
        let mut pan_curve = (n > 0).then(|| vec![0i8; n * 2]);
        let sample_runs = runs(&mut r)?;

        let mut pitch = [0i16; 128];
        let mut acc = 0i32;
        for p in &mut pitch {
            acc += r.u8()? as i32;
            *p = acc as i16;
        }
        acc = 0;
        for p in &mut pitch {
            acc += r.u8()? as i32;
            *p = (*p as i32 + (acc << 8)) as i16;
        }
        let mut samples = [0i32; 128];
        let (mut left, mut i, mut sample) = (0i32, 0usize, 0i32);
        for note in 0..128 {
            if left == 0 {
                left = next_len(&sample_runs, &mut i);
                sample = read_var(&mut r)?;
            }
            pitch[note] = (pitch[note] as i32 + (((sample - 1) & 2) << 14)) as i16;
            samples[note] = sample;
            left -= 1;
        }
        let mut exclusive = [0i8; 128];
        let (mut left, mut i, mut value, mut at) = (0i32, 0usize, 0i32, exclusive_values);
        for note in 0..128 {
            if samples[note] != 0 {
                if left == 0 {
                    left = next_len(&exclusive_runs, &mut i);
                    value = data[at] as i8 as i32 - 1;
                    at += 1;
                }
                exclusive[note] = value as i8;
                left -= 1;
            }
        }
        let mut pan = [0i8; 128];
        let (mut left, mut i, mut value, mut at) = (0i32, 0usize, 0i32, pan_values);
        for note in 0..128 {
            if samples[note] != 0 {
                if left == 0 {
                    left = next_len(&pan_runs, &mut i);
                    value = (data[at] as i8 as i32 + 16) << 2;
                    at += 1;
                }
                pan[note] = value as i8;
                left -= 1;
            }
        }
        let mut envelope = [None; 128];
        let (mut left, mut i, mut current) = (0i32, 0usize, None);
        for (note, slot) in envelope.iter_mut().enumerate() {
            if samples[note] != 0 {
                if left == 0 {
                    current = Some(envelope_of_run[i]);
                    left = next_len(&envelope_runs, &mut i);
                }
                *slot = current;
                left -= 1;
            }
        }
        let mut volume = [0i8; 128];
        let (mut left, mut i, mut value) = (0i32, 0usize, 0i32);
        for note in 0..128 {
            if left == 0 {
                left = next_len(&sample_runs, &mut i);
                if samples[note] > 0 {
                    value = r.u8()? as i32 + 1;
                }
            }
            volume[note] = value as i8;
            left -= 1;
        }
        let base_volume = r.u8()? as i32 + 1;

        for e in &mut envelopes {
            if let Some(rel) = &mut e.release {
                for k in (1..rel.len()).step_by(2) {
                    rel[k] = r.u8()? as i8;
                }
            }
            if let Some(att) = &mut e.attack {
                let end = att.len() - 2;
                for k in (3..end).step_by(2) {
                    att[k] = r.u8()? as i8;
                }
            }
        }
        for curve in [&mut volume_curve, &mut pan_curve].into_iter().flatten() {
            for k in (1..curve.len()).step_by(2) {
                curve[k] = r.u8()? as i8;
            }
        }
        for e in &mut envelopes {
            if let Some(att) = &mut e.attack {
                let mut t = 0i32;
                for k in (2..att.len()).step_by(2) {
                    t = 1 + t + r.u8()? as i32;
                    att[k] = t as i8;
                }
            }
        }
        for e in &mut envelopes {
            if let Some(rel) = &mut e.release {
                let mut t = 0i32;
                for k in (2..rel.len()).step_by(2) {
                    t += 1 + r.u8()? as i32;
                    rel[k] = t as i8;
                }
            }
        }
        if let Some(curve) = &mut volume_curve {
            read_times(&mut r, curve)?;
            let mut from = curve[0] as i32;
            let mut level = curve[1] as i32;
            let scale = |v: &mut i8, by: i32| *v = ((by * *v as i32 + 32) >> 6) as i8;
            for v in volume.iter_mut().take(from.max(0) as usize) {
                scale(v, level);
            }
            for k in (2..curve.len()).step_by(2) {
                let (to, next) = (curve[k] as i32, curve[k + 1] as i32);
                let mut acc = level * (to - from) + (to - from) / 2;
                for note in from..to {
                    let by = div_floor(acc, to - from);
                    scale(&mut volume[note as usize & 127], by);
                    acc += next - level;
                }
                from = to;
                level = next;
            }
            for v in volume.iter_mut().skip(from.max(0) as usize) {
                scale(v, level);
            }
        }
        if let Some(curve) = &mut pan_curve {
            read_times(&mut r, curve)?;
            let mut from = curve[0] as i32;
            let mut shift = (curve[1] as i32) << 1;
            let add = |v: &mut i8, by: i32| *v = (by + (*v as u8 as i32)).clamp(0, 128) as i8;
            for v in pan.iter_mut().take(from.max(0) as usize) {
                add(v, shift);
            }
            for k in (2..curve.len()).step_by(2) {
                let (to, next) = (curve[k] as i32, (curve[k + 1] as i32) << 1);
                let mut acc = shift * (to - from) + (to - from) / 2;
                for note in from..to {
                    add(&mut pan[note as usize & 127], div_floor(acc, to - from));
                    acc += next - shift;
                }
                from = to;
                shift = next;
            }
            for v in pan.iter_mut().skip(from.max(0) as usize) {
                add(v, shift);
            }
        }
        for e in &mut envelopes {
            e.decay = r.u8()? as i32;
        }
        for e in &mut envelopes {
            if e.release.is_some() {
                e.release_rate = r.u8()? as i32;
            }
            if e.attack.is_some() {
                e.attack_rate = r.u8()? as i32;
            }
            if e.decay > 0 {
                e.sustain = r.u8()? as i32;
            }
        }
        for e in &mut envelopes {
            e.vibrato_depth = r.u8()? as i32;
        }
        for e in &mut envelopes {
            if e.vibrato_depth > 0 {
                e.vibrato_rate = r.u8()? as i32;
            }
        }
        for e in &mut envelopes {
            if e.vibrato_rate > 0 {
                e.vibrato_delay = r.u8()? as i32;
            }
        }
        Ok(Patch { samples, pitch, volume, pan, exclusive, envelope, envelopes, base_volume })
    }
}

/// The time column of a volume or pan curve: first an absolute note, then steps of 1 + n.
fn read_times(r: &mut Reader, curve: &mut [i8]) -> Result<()> {
    let mut t = r.u8()? as i32;
    curve[0] = t as i8;
    for k in (2..curve.len()).step_by(2) {
        t = 1 + t + r.u8()? as i32;
        curve[k] = t as i8;
    }
    Ok(())
}

/// `ByteBuf.method13099`.
fn read_var(r: &mut Reader) -> Result<i32> {
    let mut v = 0i32;
    loop {
        let b = r.u8()?;
        if b & 0x80 == 0 {
            return Ok(v | b as i32);
        }
        v = (v | (b & 0x7f) as i32) << 7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_toward_negative_infinity() {
        assert_eq!(div_floor(7, 2), 3);
        assert_eq!(div_floor(-7, 2), -4);
    }
}
