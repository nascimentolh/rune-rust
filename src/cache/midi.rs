//! Reading a standard MIDI file the way the client does. Port of `Class80` (track cursors,
//! running status — also for meta events —, tempo changes; the playback timing parts
//! `method1403`/`method1406` come with the player) and `Node_Sub7.method12165` (which
//! notes each program plays, so only those samples are loaded).

use std::collections::BTreeMap;

/// `Class80.aByteArray784`: data bytes after each status (0x80..=0xff).
const DATA_BYTES: [u8; 128] = {
    let mut t = [0u8; 128];
    let mut i = 0;
    while i < 128 {
        t[i] = match i + 128 {
            0x80..=0xbf => 2,
            0xc0..=0xdf => 1,
            0xe0..=0xef => 2,
            0xf1 | 0xf3 => 1,
            0xf2 => 2,
            _ => 0,
        };
        i += 1;
    }
    t
};

/// What `next_event` read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// A channel message packed as status | data1 << 8 | data2 << 16.
    Message(i32),
    /// End of track (meta 47).
    End,
    /// Tempo change (meta 81), already applied.
    Tempo,
    /// Other meta or a sysex, skipped.
    Skipped,
}

/// `Class80`.
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    /// `anInt790`: microseconds per quarter note.
    pub tempo: i32,
    /// `aLong786`: accumulated time base.
    pub time_base: i64,
    /// `anIntArray787`: each track's read position, `None` once it ended.
    cursors: Vec<Option<usize>>,
    /// `anIntArray788`: each track's tick of its next event.
    pub ticks: Vec<i32>,
    /// `anIntArray796`: running status per track.
    running: Vec<i32>,
}

impl<'a> Reader<'a> {
    /// `method1394`.
    pub fn new(data: &'a [u8]) -> Self {
        let be16 = |at: usize| data.get(at..at + 2).map_or(0, |b| u16::from_be_bytes([b[0], b[1]]) as usize);
        let be32 = |at: usize| data.get(at..at + 4).map_or(0, |b| i32::from_be_bytes([b[0], b[1], b[2], b[3]]));
        let tracks = be16(10);
        let mut starts = Vec::with_capacity(tracks);
        let mut at = 14;
        while starts.len() < tracks && at + 8 <= data.len() {
            let (kind, len) = (be32(at), be32(at + 4));
            at += 8;
            if kind == 0x4d54_726b {
                starts.push(at);
            }
            at += len.max(0) as usize;
        }
        let n = starts.len();
        Reader {
            data,
            pos: 0,
            tempo: 500_000,
            time_base: 0,
            cursors: starts.iter().map(|&s| Some(s)).collect(),
            ticks: vec![0; n],
            running: vec![0; n],
        }
    }

    pub fn tracks(&self) -> usize {
        self.cursors.len()
    }

    fn byte(&mut self) -> i32 {
        let b = self.data.get(self.pos).copied().unwrap_or(0) as i32;
        self.pos += 1;
        b
    }

    fn var(&mut self) -> i32 {
        let mut v = 0i32;
        loop {
            let b = self.byte();
            if b & 0x80 == 0 {
                return v | b;
            }
            v = (v | (b & 0x7f)) << 7;
        }
    }

    /// `method1398`: continue reading track `t`.
    pub fn seek(&mut self, t: usize) {
        self.pos = self.cursors[t].unwrap_or(usize::MAX);
    }

    /// `method1404`: remember where track `t` is.
    pub fn save(&mut self, t: usize) {
        self.cursors[t] = (self.pos != usize::MAX).then_some(self.pos);
    }

    /// `method1400`: track ended.
    pub fn end_track(&mut self) {
        self.pos = usize::MAX;
    }

    /// `method1392`: adds the next delta time to track `t`.
    pub fn read_delta(&mut self, t: usize) {
        let d = self.var();
        self.ticks[t] = self.ticks[t].wrapping_add(d);
    }

    /// `method1393`: the live track with the earliest next event.
    pub fn earliest(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        for t in 0..self.cursors.len() {
            if self.cursors[t].is_some() && best.is_none_or(|b| self.ticks[t] < self.ticks[b]) {
                best = Some(t);
            }
        }
        best
    }

    /// `method1405`: every track has ended.
    pub fn finished(&self) -> bool {
        self.cursors.iter().all(Option::is_none)
    }

    /// `method1423` / `method1401`: the next event of track `t` (call `seek` first).
    pub fn next_event(&mut self, t: usize) -> Event {
        let first = self.data.get(self.pos).copied().unwrap_or(0) as i32;
        let status = if first >= 0x80 {
            self.running[t] = first;
            self.pos += 1;
            first
        } else {
            self.running[t]
        };
        if status == 0xf0 || status == 0xf7 {
            let len = self.var();
            if status == 0xf7 && len > 0 {
                let next = self.data.get(self.pos).copied().unwrap_or(0) as i32;
                if matches!(next, 241..=243 | 246 | 248 | 250..=252 | 254) {
                    self.pos += 1;
                    self.running[t] = next;
                    return self.message(t, next);
                }
            }
            self.pos += len.max(0) as usize;
            return Event::Skipped;
        }
        self.message(t, status)
    }

    /// `method1412`.
    fn message(&mut self, t: usize, status: i32) -> Event {
        if status == 0xff {
            let kind = self.byte();
            let mut len = self.var();
            match kind {
                47 => {
                    self.pos += len.max(0) as usize;
                    Event::End
                }
                81 => {
                    let tempo = self.byte() << 16 | self.byte() << 8 | self.byte();
                    len -= 3;
                    self.time_base += self.ticks[t] as i64 * (self.tempo - tempo) as i64;
                    self.tempo = tempo;
                    self.pos += len.max(0) as usize;
                    Event::Tempo
                }
                _ => {
                    self.pos += len.max(0) as usize;
                    Event::Skipped
                }
            }
        } else {
            let n = DATA_BYTES[(status - 128).clamp(0, 127) as usize];
            let mut v = status;
            if n >= 1 {
                v |= self.byte() << 8;
            }
            if n >= 2 {
                v |= self.byte() << 16;
            }
            Event::Message(v)
        }
    }
}

/// `Node_Sub7.method12165`: for each program (bank << 7 | program; channel 9 starts at 128,
/// the drum bank) the notes the music plays with a velocity above 0.
pub fn used_notes(midi: &[u8]) -> BTreeMap<i32, [bool; 128]> {
    let mut used: BTreeMap<i32, [bool; 128]> = BTreeMap::new();
    let mut bank = [0i32; 16];
    let mut program = [0i32; 16];
    bank[9] = 128;
    program[9] = 128;
    let mut r = Reader::new(midi);
    for t in 0..r.tracks() {
        r.seek(t);
        r.read_delta(t);
        r.save(t);
    }
    while !r.finished() {
        let Some(t) = r.earliest() else { break };
        let tick = r.ticks[t];
        while tick == r.ticks[t] {
            r.seek(t);
            let event = r.next_event(t);
            if event == Event::End {
                r.end_track();
                r.save(t);
                break;
            }
            if let Event::Message(m) = event {
                let (kind, ch) = (m & 0xf0, (m & 0xf) as usize);
                let (d1, d2) = (m >> 8 & 0x7f, m >> 16 & 0x7f);
                match kind {
                    0xb0 if d1 == 0 => bank[ch] = (d2 << 14) + (bank[ch] & !0x1f_c000),
                    0xb0 if d1 == 32 => bank[ch] = (bank[ch] & !0x3f80) + (d2 << 7),
                    0xc0 => program[ch] = d1 + bank[ch],
                    0x90 if d2 > 0 => used.entry(program[ch]).or_insert([false; 128])[d1 as usize] = true,
                    _ => {}
                }
            }
            r.read_delta(t);
            r.save(t);
        }
    }
    used
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One track: program 5 on channel 0, then a note on 60 and the end.
    fn sample() -> Vec<u8> {
        let mut d = b"MThd\0\0\0\x06\0\0\0\x01\0\x60MTrk".to_vec();
        let body = [0x00, 0xc0, 5, 0x00, 0x90, 60, 100, 0x60, 0xff, 0x2f, 0x00];
        d.extend((body.len() as u32).to_be_bytes());
        d.extend(body);
        d
    }

    #[test]
    fn finds_the_notes_each_program_plays() {
        let used = used_notes(&sample());
        assert_eq!(used.keys().copied().collect::<Vec<_>>(), vec![5]);
        assert!(used[&5][60] && !used[&5][61]);
    }

    #[test]
    fn data_byte_table_matches_the_reference() {
        assert_eq!(
            (DATA_BYTES[0x00], DATA_BYTES[0x40], DATA_BYTES[0x60], DATA_BYTES[0x71], DATA_BYTES[0x72], DATA_BYTES[0x7f]),
            (2, 1, 2, 1, 2, 0)
        );
    }
}
