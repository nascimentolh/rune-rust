//! The MIDI music player. Port of `Node_Sub15_Sub2` (song timing, MIDI events, channels,
//! note volume/pitch/pan, envelopes), `Node_Sub15_Sub3` (mixing the notes in 10 ms steps),
//! `Node_Sub15_Sub4` (notes fading out) and `Node_Sub13` (one sounding note). Samples are
//! mixed by `voice::Voice`; the result is 22,050 Hz like `Class253`.
//!
//! ponytail: retrigger (controller 81, `Node_Sub15_Sub3.method15234`) and the output's voice
//! budget (`Class253.method4337` priorities) aren't ported yet.

pub mod voice;

use crate::cache::midi::{Event, Reader};
use crate::cache::music_patch::{Envelope, Patch};
use std::collections::HashMap;
use std::rc::Rc;
use voice::{SampleData, Voice};

/// `Class253.anInt3129`.
pub const RATE: i32 = 22050;

/// A program's patch with its loaded samples (`Node_Sub14.aNode_Sub26_Sub1Array7589`).
pub struct LoadedPatch {
    pub patch: Patch,
    pub samples: Vec<Option<Rc<SampleData>>>,
}

/// `Node_Sub13`.
struct Note {
    id: u64,
    channel: usize,
    sample: Rc<SampleData>,
    envelope: Envelope,
    /// `anInt7567`, `anInt7569`, `anInt7570`, `anInt7572`, `anInt7573`.
    exclusive: i32,
    key: i32,
    volume: i32,
    pan: i32,
    pitch: i32,
    /// `anInt7574` / `anInt7575`: portamento distance and remaining fraction (4096 = all).
    glide: i32,
    glide_left: i32,
    /// `anInt7576`: decay time; `anInt7577`/`anInt7578`: envelope time and point;
    /// `anInt7579`/`anInt7580`: release time (-1 while held) and point.
    decay: i32,
    env_time: i32,
    env_point: usize,
    release_time: i32,
    release_point: usize,
    /// `anInt7581`: 10 ms ticks; `anInt7571`: vibrato phase; `anInt7584`: samples to next update.
    ticks: i32,
    vibrato: i32,
    until_update: i32,
    voice: Option<Voice>,
    /// Unlinked from the note list.
    removed: bool,
}

pub struct Player {
    stereo: bool,
    reader: Option<Reader>,
    looping: bool,
    /// `volume` (`setVolume`), `anInt9690`, `anInt9695`.
    volume: i32,
    master: i32,
    tempo_scale: i64,
    channel_master: [i32; 16],
    channel_volume: [i32; 16],
    channel_pan: [i32; 16],
    expression: [i32; 16],
    bend: [i32; 16],
    modulation: [i32; 16],
    portamento_time: [i32; 16],
    bank: [i32; 16],
    program: [i32; 16],
    default_program: [i32; 16],
    flags: [i32; 16],
    rpn: [i32; 16],
    bend_range: [i32; 16],
    sample_offset: [i32; 16],
    active: [[Option<u64>; 128]; 16],
    exclusive: [[Option<u64>; 128]; 16],
    notes: Vec<Note>,
    fading: Vec<Voice>,
    pub patches: HashMap<i32, LoadedPatch>,
    next_id: u64,
    /// `aLong9710`, `anInt9717`, `anInt9696`, `aLong9693`.
    time: i64,
    next_track: usize,
    next_tick: i32,
    next_time: i64,
}

impl Player {
    pub fn new(stereo: bool) -> Self {
        let mut p = Player {
            stereo,
            reader: None,
            looping: false,
            volume: 256,
            master: 256,
            tempo_scale: 1_000_000,
            channel_master: [256; 16],
            channel_volume: [0; 16],
            channel_pan: [0; 16],
            expression: [0; 16],
            bend: [0; 16],
            modulation: [0; 16],
            portamento_time: [0; 16],
            bank: [0; 16],
            program: [0; 16],
            default_program: [0; 16],
            flags: [0; 16],
            rpn: [0; 16],
            bend_range: [0; 16],
            sample_offset: [0; 16],
            active: [[None; 128]; 16],
            exclusive: [[None; 128]; 16],
            notes: Vec::new(),
            fading: Vec::new(),
            patches: HashMap::new(),
            next_id: 1,
            time: 0,
            next_track: 0,
            next_tick: 0,
            next_time: 0,
        };
        p.reset(true);
        // `method15137` → `method15105`: channel 10 plays the drum bank.
        p.default_program[9] = 128;
        p.bank[9] = 128;
        p.set_program(9, 128);
        p
    }

    /// `setVolume` (0..=256 scale of the music preference).
    pub fn set_volume(&mut self, v: i32) {
        self.volume = v;
    }

    /// `method15131`: start a song (MIDI bytes).
    pub fn play(&mut self, midi: &[u8], looping: bool) {
        self.reset(true);
        let mut r = Reader::new(midi);
        self.looping = looping;
        self.time = 0;
        for t in 0..r.tracks() {
            r.seek(t);
            r.read_delta(t);
            r.save(t);
        }
        let Some(t) = r.earliest() else { return };
        self.next_track = t;
        self.next_tick = r.ticks[t];
        self.next_time = r.time_of(self.next_tick);
        self.reader = Some(r);
    }

    pub fn playing(&self) -> bool {
        self.reader.is_some()
    }

    /// `method15116`.
    fn reset(&mut self, all_sound_off: bool) {
        if all_sound_off {
            self.sound_off(None);
        } else {
            self.notes_off(None);
        }
        for ch in 0..16 {
            self.reset_controllers(ch);
        }
        self.program = self.default_program;
        for ch in 0..16 {
            self.bank[ch] = self.default_program[ch] & !127;
        }
    }

    /// `method15114`.
    fn reset_controllers(&mut self, ch: usize) {
        self.channel_volume[ch] = 12800;
        self.channel_pan[ch] = 8192;
        self.expression[ch] = 16383;
        self.bend[ch] = 8192;
        self.modulation[ch] = 0;
        self.portamento_time[ch] = 8192;
        self.end_portamento(ch);
        self.flags[ch] = 0;
        self.rpn[ch] = 32767;
        self.bend_range[ch] = 256;
        self.sample_offset[ch] = 0;
    }

    /// `method15189`.
    fn set_program(&mut self, ch: usize, program: i32) {
        if program != self.program[ch] {
            self.program[ch] = program;
            self.exclusive[ch] = [None; 128];
        }
    }

    /// `method15117`: portamento off lets go of glided notes.
    fn end_portamento(&mut self, ch: usize) {
        if self.flags[ch] & 2 != 0 {
            for n in &mut self.notes {
                if n.channel == ch && self.active[ch][n.key as usize].is_none() && n.release_time < 0 {
                    n.release_time = 0;
                }
            }
        }
    }

    /// `method15111`: all sound off (fade every voice in 10 ms).
    fn sound_off(&mut self, ch: Option<usize>) {
        let mut kept = Vec::new();
        for mut n in std::mem::take(&mut self.notes) {
            if ch.is_some_and(|c| c != n.channel) {
                kept.push(n);
                continue;
            }
            if let Some(mut v) = n.voice.take() {
                v.fade_out(RATE / 100);
                if v.ramping() {
                    self.fading.push(v);
                }
            }
            if n.release_time < 0 {
                self.active[n.channel][n.key as usize] = None;
            }
        }
        self.notes = kept;
    }

    /// `method15192`: all notes off (release them).
    fn notes_off(&mut self, ch: Option<usize>) {
        for n in &mut self.notes {
            if ch.is_none_or(|c| c == n.channel) && n.release_time < 0 {
                self.active[n.channel][n.key as usize] = None;
                n.release_time = 0;
            }
        }
    }

    /// `method15109`: note off.
    fn note_off(&mut self, ch: usize, key: usize) {
        let Some(id) = self.active[ch][key].take() else { return };
        let gliding = self.flags[ch] & 2 != 0;
        let other_held = self.notes.iter().any(|o| o.channel == ch && o.release_time < 0 && o.id != id);
        if let Some(n) = self.notes.iter_mut().find(|n| n.id == id) {
            if !gliding || other_held {
                n.release_time = 0;
            }
        }
    }

    /// `method15107`: note on.
    fn note_on(&mut self, ch: usize, key: usize, velocity: i32) {
        self.note_off(ch, key);
        if self.flags[ch] & 2 != 0 {
            if let Some(n) = self.notes.iter_mut().rev().find(|n| n.channel == ch && n.release_time < 0) {
                self.active[ch][n.key as usize] = None;
                self.active[ch][key] = Some(n.id);
                let now = ((n.glide_left * n.glide) >> 12) + n.pitch;
                n.pitch += (key as i32 - n.key) << 8;
                n.glide = now - n.pitch;
                n.glide_left = 4096;
                n.key = key as i32;
                return;
            }
        }
        let program = self.program[ch];
        let Some(lp) = self.patches.get(&program) else { return };
        let Some(sample) = lp.samples.get(key).cloned().flatten() else { return };
        let p = &lp.patch;
        let envelope = p.envelope[key].and_then(|e| p.envelopes.get(e)).cloned().unwrap_or_default();
        let pitch_short = p.pitch[key];
        let mut note = Note {
            id: self.next_id,
            channel: ch,
            sample: Rc::clone(&sample),
            envelope,
            exclusive: p.exclusive[key] as i32,
            key: key as i32,
            volume: (velocity * velocity * p.volume[key] as i32 * p.base_volume + 1024) >> 11,
            pan: p.pan[key] as u8 as i32,
            pitch: ((key as i32) << 8) - (pitch_short as i32 & 0x7fff),
            glide: 0,
            glide_left: 0,
            decay: 0,
            env_time: 0,
            env_point: 0,
            release_time: -1,
            release_point: 0,
            ticks: 0,
            vibrato: 0,
            until_update: 0,
            voice: None,
            removed: false,
        };
        self.next_id += 1;
        let rate = self.rate_of(&note);
        let pan = self.pan_of(&note);
        if !sample.pcm.is_empty() {
            let vol = if self.sample_offset[ch] == 0 { self.volume_of(&note) } else { 0 };
            let mut v = Voice::new(Rc::clone(&sample), rate, vol, pan);
            if self.sample_offset[ch] != 0 {
                self.seek_offset(&mut v, ch, pitch_short < 0);
            }
            if pitch_short < 0 {
                v.loops = -1;
            }
            note.voice = Some(v);
        }
        if note.exclusive >= 0 {
            let class = note.exclusive as usize & 127;
            if let Some(prev) = self.exclusive[ch][class] {
                if let Some(o) = self.notes.iter_mut().find(|o| o.id == prev && o.release_time < 0) {
                    self.active[ch][o.key as usize] = None;
                    o.release_time = 0;
                }
            }
            self.exclusive[ch][class] = Some(note.id);
        }
        self.active[ch][key] = Some(note.id);
        self.notes.push(note);
    }

    /// `method15099`: start the sample part-way (controller 16/48).
    fn seek_offset(&self, v: &mut Voice, ch: usize, looped: bool) {
        let s = Rc::clone(v.sample());
        let mut len = s.pcm.len() as i32;
        let pos = if looped && s.ping_pong {
            let span = len + len - s.loop_start;
            let mut p = ((self.sample_offset[ch] as i64 * span as i64) >> 6) as i32;
            len <<= 8;
            if p >= len {
                p = len + len - 1 - p;
                v.reverse();
            }
            p
        } else {
            ((self.sample_offset[ch] as i64 * len as i64) >> 6) as i32
        };
        v.set_position(pos);
    }

    /// `method15108`: the note's volume now.
    fn volume_of(&self, n: &Note) -> i32 {
        let ch = n.channel;
        if self.channel_master[ch] == 0 {
            return 0;
        }
        let e = &n.envelope;
        let mut v = (self.expression[ch] * self.channel_volume[ch] + 4096) >> 13;
        v = (v * v + 16384) >> 15;
        v = (v * n.volume + 16384) >> 15;
        v = (v * self.volume + 128) >> 8;
        v = (v * self.master) >> 8;
        v = (v * self.channel_master[ch] + 128) >> 8;
        if e.decay > 0 {
            v = (v as f64 * 0.5f64.powf(n.decay as f64 * 1.953125E-5 * e.decay as f64) + 0.5) as i32;
        }
        if let Some(curve) = &e.envelope {
            let mut level = curve[n.env_point + 1] as i32;
            if n.env_point < curve.len() - 2 {
                let t0 = (curve[n.env_point] as u8 as i32) << 8;
                let t1 = (curve[n.env_point + 2] as u8 as i32) << 8;
                level += (n.env_time - t0) * (curve[n.env_point + 3] as i32 - level) / (t1 - t0);
            }
            v = (v * level + 32) >> 6;
        }
        if n.release_time > 0 {
            if let Some(curve) = &e.release {
                let mut level = curve[n.release_point + 1] as i32;
                if n.release_point < curve.len() - 2 {
                    let t0 = (curve[n.release_point] as u8 as i32) << 8;
                    let t1 = (curve[n.release_point + 2] as u8 as i32) << 8;
                    level += (curve[n.release_point + 3] as i32 - level) * (n.release_time - t0) / (t1 - t0);
                }
                v = (level * v + 32) >> 6;
            }
        }
        v
    }

    /// `method15120`: the note's playback rate now.
    fn rate_of(&self, n: &Note) -> i32 {
        let ch = n.channel;
        let mut pitch = ((n.glide * n.glide_left) >> 12) + n.pitch;
        pitch += ((self.bend[ch] - 8192) * self.bend_range[ch]) >> 12;
        let e = &n.envelope;
        if e.vibrato_speed > 0 && (e.vibrato_depth > 0 || self.modulation[ch] > 0) {
            let mut depth = e.vibrato_depth << 2;
            let delay = e.vibrato_delay << 1;
            if n.ticks < delay {
                depth = depth * n.ticks / delay;
            }
            depth += self.modulation[ch] >> 7;
            pitch += ((0.01227184630308513 * (n.vibrato & 0x1ff) as f64).sin() * depth as f64) as i32;
        }
        let rate = ((n.sample.rate * 256) as f64 * 2f64.powf(3.255208333333333E-4 * pitch as f64) / RATE as f64 + 0.5) as i32;
        rate.max(1)
    }

    /// `method15122`.
    fn pan_of(&self, n: &Note) -> i32 {
        let p = self.channel_pan[n.channel];
        if p < 8192 {
            (p * n.pan + 32) >> 6
        } else {
            16384 - (((128 - n.pan) * (16384 - p) + 32) >> 6)
        }
    }

    /// `method15119`: one MIDI message.
    fn message(&mut self, m: i32) {
        let ch = (m & 0xf) as usize;
        let d1 = m >> 8 & 0x7f;
        let d2 = m >> 16 & 0x7f;
        let msb = |v: i32| (d2 << 7) + (v & -16257);
        let lsb = |v: i32| d2 + (v & -128);
        match m & 0xf0 {
            0x80 => self.note_off(ch, d1 as usize),
            0x90 if d2 > 0 => self.note_on(ch, d1 as usize, d2),
            0x90 => self.note_off(ch, d1 as usize),
            0xb0 => match d1 {
                0 => self.bank[ch] = (d2 << 14) + (self.bank[ch] & -2080769),
                32 => self.bank[ch] = (d2 << 7) + (self.bank[ch] & -16257),
                1 => self.modulation[ch] = msb(self.modulation[ch]),
                33 => self.modulation[ch] = lsb(self.modulation[ch]),
                5 => self.portamento_time[ch] = msb(self.portamento_time[ch]),
                37 => self.portamento_time[ch] = lsb(self.portamento_time[ch]),
                7 => self.channel_volume[ch] = msb(self.channel_volume[ch]),
                39 => self.channel_volume[ch] = lsb(self.channel_volume[ch]),
                10 => self.channel_pan[ch] = msb(self.channel_pan[ch]),
                42 => self.channel_pan[ch] = lsb(self.channel_pan[ch]),
                11 => self.expression[ch] = msb(self.expression[ch]),
                43 => self.expression[ch] = lsb(self.expression[ch]),
                64 => {
                    if d2 >= 64 {
                        self.flags[ch] |= 1;
                    } else {
                        self.flags[ch] &= !1;
                    }
                }
                65 => {
                    if d2 >= 64 {
                        self.flags[ch] |= 2;
                    } else {
                        self.end_portamento(ch);
                        self.flags[ch] &= !2;
                    }
                }
                99 => self.rpn[ch] = (d2 << 7) + (self.rpn[ch] & 0x7f),
                98 => self.rpn[ch] = (self.rpn[ch] & 0x3f80) + d2,
                101 => self.rpn[ch] = (d2 << 7) + (self.rpn[ch] & 0x7f) + 16384,
                100 => self.rpn[ch] = (self.rpn[ch] & 0x3f80) + d2 + 16384,
                120 => self.sound_off(Some(ch)),
                121 => self.reset_controllers(ch),
                123 => self.notes_off(Some(ch)),
                6 if self.rpn[ch] == 16384 => self.bend_range[ch] = msb(self.bend_range[ch]),
                38 if self.rpn[ch] == 16384 => self.bend_range[ch] = lsb(self.bend_range[ch]),
                16 => self.sample_offset[ch] = msb(self.sample_offset[ch]),
                48 => self.sample_offset[ch] = lsb(self.sample_offset[ch]),
                81 => {
                    if d2 >= 64 {
                        self.flags[ch] |= 4;
                    } else {
                        self.flags[ch] &= !4;
                    }
                }
                _ => {}
            },
            0xc0 => self.set_program(ch, d1 + self.bank[ch]),
            0xe0 => self.bend[ch] = (m >> 8 & 0x7f) + (m >> 9 & 0x3f80),
            _ => {
                if m & 0xff == 0xff {
                    self.reset(true);
                }
            }
        }
    }

    /// `method15127`: every event due at the current tick.
    fn events(&mut self) {
        let Some(mut r) = self.reader.take() else { return };
        let (mut t, mut tick, mut time) = (self.next_track, self.next_tick, self.next_time);
        loop {
            if tick != self.next_tick {
                (self.next_track, self.next_tick, self.next_time) = (t, tick, time);
                break;
            }
            while tick == r.ticks[t] {
                r.seek(t);
                match r.next_event(t) {
                    Event::End => {
                        r.end_track();
                        r.save(t);
                        if r.finished() {
                            if !self.looping || tick == 0 {
                                self.reset(true);
                                return;
                            }
                            r.rewind(time);
                        }
                        break;
                    }
                    Event::Message(m) if m & 0x80 != 0 => self.message(m),
                    _ => {}
                }
                r.read_delta(t);
                r.save(t);
            }
            let Some(next) = r.earliest() else { break };
            t = next;
            tick = r.ticks[t];
            time = r.time_of(tick);
        }
        self.reader = Some(r);
    }

    /// `method12230`: adds `n` frames to `out` (interleaved when stereo).
    pub fn render(&mut self, out: &mut [i32], mut at: usize, mut n: usize) {
        if let Some(division) = self.reader.as_ref().map(|r| r.division) {
            let step = division as i64 * self.tempo_scale / RATE as i64;
            while step > 0 && self.reader.is_some() {
                let t = step * n as i64 + self.time;
                if self.next_time - t >= 0 {
                    self.time = t;
                    break;
                }
                let k = ((self.next_time - self.time + step - 1) / step) as usize;
                self.time += k as i64 * step;
                self.mix(out, at, k);
                at += k;
                n -= k;
                self.events();
            }
        }
        self.mix(out, at, n);
    }

    /// `Node_Sub15_Sub3.method12230`: the fading voices, then every note in 10 ms steps.
    fn mix(&mut self, out: &mut [i32], at: usize, n: usize) {
        let stereo = self.stereo;
        for v in &mut self.fading {
            v.mix(out, at, n, stereo);
        }
        self.fading.retain(|v| !v.done);
        let mut notes = std::mem::take(&mut self.notes);
        let mut kept = Vec::with_capacity(notes.len());
        for mut note in notes.drain(..) {
            // `method15115`: a note without a voice ends once released.
            if note.voice.is_none() {
                if note.release_time >= 0 {
                    self.forget(&note);
                    continue;
                }
                kept.push(note);
                continue;
            }
            let (mut a, mut left) = (at, n);
            loop {
                if left as i32 <= note.until_update {
                    if let Some(v) = &mut note.voice {
                        v.mix(out, a, left, stereo);
                    }
                    note.until_update -= left as i32;
                    break;
                }
                let step = note.until_update.max(0) as usize;
                if let Some(v) = &mut note.voice {
                    v.mix(out, a, step, stereo);
                }
                a += step;
                left -= step;
                if self.update(&mut note, out, a, left) {
                    break;
                }
            }
            if !note.removed {
                kept.push(note);
            }
        }
        kept.append(&mut self.notes);
        self.notes = kept;
    }

    fn forget(&mut self, n: &Note) {
        if n.exclusive > 0 && self.exclusive[n.channel][n.exclusive as usize & 127] == Some(n.id) {
            self.exclusive[n.channel][n.exclusive as usize & 127] = None;
        }
    }

    /// `method15128`: the 10 ms update; true when its sound has ended (the note leaves the list
    /// only if it was let go: `removed`).
    fn update(&mut self, n: &mut Note, out: &mut [i32], at: usize, left: usize) -> bool {
        n.until_update = RATE / 100;
        let held_or_sounding = n.release_time < 0 || n.voice.as_ref().is_some_and(|v| !v.outside());
        if !held_or_sounding {
            n.voice = None;
            n.removed = true;
            self.forget(n);
            return true;
        }
        if n.glide_left > 0 {
            let fall = (16.0 * 2f64.powf(self.portamento_time[n.channel] as f64 * 4.921259842519685E-4) + 0.5) as i32;
            n.glide_left = (n.glide_left - fall).max(0);
        }
        let rate = self.rate_of(n);
        if let Some(v) = &mut n.voice {
            v.set_rate(rate);
        }
        let e = n.envelope.clone();
        let mut ended = false;
        n.ticks += 1;
        n.vibrato += e.vibrato_speed;
        let d = 5.086263020833333E-6 * (((n.key - 60) << 8) + ((n.glide * n.glide_left) >> 12)) as f64;
        let speed = |rate: i32| if rate > 0 { (128.0 * 2f64.powf(d * rate as f64) + 0.5) as i32 } else { 128 };
        if e.decay > 0 {
            n.decay += speed(e.decay_rate);
            if e.decay * n.decay >= 819200 {
                ended = true;
            }
        }
        if let Some(curve) = &e.envelope {
            n.env_time += speed(e.envelope_rate);
            while n.env_point < curve.len() - 2 && n.env_time > (curve[n.env_point + 2] as u8 as i32) << 8 {
                n.env_point += 2;
            }
            if curve.len() - 2 == n.env_point && curve[n.env_point + 1] == 0 {
                ended = true;
            }
        }
        let sustained = self.flags[n.channel] & 1 != 0;
        let exclusive_head = n.exclusive >= 0 && self.exclusive[n.channel][n.exclusive as usize & 127] == Some(n.id);
        if n.release_time >= 0 && !sustained && !exclusive_head {
            if let Some(curve) = &e.release {
                n.release_time += speed(e.release_rate);
                while n.release_point < curve.len() - 2 && n.release_time > (curve[n.release_point + 2] as u8 as i32) << 8 {
                    n.release_point += 2;
                }
                if curve.len() - 2 == n.release_point {
                    ended = true;
                }
            }
        }
        if ended {
            if let Some(mut v) = n.voice.take() {
                v.fade_out(n.until_update);
                v.mix(out, at, left, self.stereo);
                if v.ramping() {
                    self.fading.push(v);
                }
            }
            if n.release_time >= 0 {
                n.removed = true;
                self.forget(n);
            }
            return true;
        }
        let (vol, pan) = (self.volume_of(n), self.pan_of(n));
        if let Some(v) = &mut n.voice {
            v.ramp_to(n.until_update, vol, pan);
        }
        false
    }
}

/// Renders `frames` frames of the player into 16-bit PCM (`Class253_Sub1.method4342`: the
/// int mix clamps to 24 bits, then keeps its top 16).
pub fn to_pcm16(mix: &[i32]) -> Vec<i16> {
    mix.iter()
        .map(|&v| {
            let v = if (v + 8_388_608) & -16_777_216 != 0 { 0x7f_ffff ^ (v >> 31) } else { v };
            (v >> 8) as i16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm_conversion_clamps_like_the_reference() {
        assert_eq!(to_pcm16(&[256, -256, 9_000_000, -9_000_000]), vec![1, -1, 32767, -32768]);
    }

    #[test]
    fn an_empty_player_renders_silence() {
        let mut p = Player::new(true);
        let mut out = vec![0i32; 512];
        p.render(&mut out, 0, 256);
        assert!(out.iter().all(|&v| v == 0) && !p.playing());
    }
}
