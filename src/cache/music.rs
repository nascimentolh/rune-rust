//! Music tracks (index 6) and the login music's id. Port of `Node_Sub7`'s constructor, which
//! rebuilds a standard MIDI file from the cache's packed form, and of `Class89.method1496`
//! (defaults file 4, opcode 1: the login screen's track).
//!
//! The packed form keeps each kind of value in its own run: event codes (status nibble in the
//! low 4 bits, channel change in the high 4), delta times, then per kind the note numbers,
//! velocities, controller numbers and values, pitch bends, aftertouch and tempos. Values are
//! deltas from the previous of their kind. The trailer holds the track count and division.

use super::buffer::Reader;
use super::error::{corrupt, Result};

/// `Class89.method1496`: opcode 1 gives the login music track (the first one wins).
pub fn login_track(data: &[u8]) -> Result<Option<u32>> {
    let mut r = Reader::new(data);
    let mut track = None;
    loop {
        match r.u8()? {
            0 => return Ok(track),
            1 => {
                let id = r.u16()? as u32;
                track.get_or_insert(id);
            }
            // Other opcodes carry nothing (the reference reads none for them).
            _ => {}
        }
    }
}

/// `ByteBuf.method13099`: a big-endian 7-bit variable-length number.
fn var_int(data: &[u8], at: &mut usize) -> Result<u32> {
    let mut v = 0u32;
    loop {
        let b = *data.get(*at).ok_or(super::error::CacheError::Corrupt("music track ends early".into()))?;
        *at += 1;
        if b & 0x80 == 0 {
            return Ok(v | b as u32);
        }
        v = (v | (b & 0x7f) as u32) << 7;
    }
}

/// `ByteBuf.method13077`: writes a MIDI variable-length number.
fn put_var(out: &mut Vec<u8>, v: u32) {
    for shift in [28, 21, 14, 7] {
        if v >> shift != 0 {
            out.push((v >> shift) as u8 | 0x80);
        }
    }
    out.push((v & 0x7f) as u8);
}

/// A run of bytes inside the packed track, read one after another.
struct Run<'a> {
    data: &'a [u8],
    at: usize,
}

impl Run<'_> {
    fn next(&mut self) -> Result<i32> {
        let b = *self.data.get(self.at).ok_or(super::error::CacheError::Corrupt("music track ends early".into()))?;
        self.at += 1;
        Ok(b as i8 as i32)
    }
}

/// Which run a controller's values are in (`Node_Sub7`'s controller split).
fn controller_run(controller: i32) -> usize {
    match controller {
        0 | 32 => BANK,
        1 => MOD_MSB,
        33 => MOD_LSB,
        7 => VOLUME_MSB,
        39 => VOLUME_LSB,
        10 => PAN_MSB,
        42 => PAN_LSB,
        99 => NRPN_MSB,
        98 => NRPN_LSB,
        101 => RPN_MSB,
        100 => RPN_LSB,
        64 | 65 | 120 | 121 | 123 => SWITCHES,
        _ => OTHER_CONTROLLERS,
    }
}

// Runs, in the order they follow the controller numbers in the file.
const SWITCHES: usize = 0;
const AFTERTOUCH_VALUES: usize = 1;
const CHANNEL_PRESSURE: usize = 2;
const BEND_HIGH: usize = 3;
const MOD_MSB: usize = 4;
const VOLUME_MSB: usize = 5;
const PAN_MSB: usize = 6;
const NOTES: usize = 7;
const NOTE_ON_VELOCITY: usize = 8;
const OTHER_CONTROLLERS: usize = 9;
const NOTE_OFF_VELOCITY: usize = 10;
const MOD_LSB: usize = 11;
const VOLUME_LSB: usize = 12;
const PAN_LSB: usize = 13;
/// Bank selects and program changes share one run.
const BANK: usize = 14;
const BEND_LOW: usize = 15;
const NRPN_MSB: usize = 16;
const NRPN_LSB: usize = 17;
const RPN_MSB: usize = 18;
const RPN_LSB: usize = 19;
const TEMPOS: usize = 20;

/// Rebuilds the standard MIDI file (format 0 or 1) of a packed track.
pub fn to_midi(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 3 {
        return corrupt("music track too short");
    }
    let tracks = data[data.len() - 3] as usize;
    let division = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]);

    // Pass 1: count each event kind (the event codes come first).
    let mut at = 0;
    let (mut tempos, mut note_on, mut note_off, mut controllers, mut bends, mut pressure, mut aftertouch, mut programs) =
        (0, 0, 0, 0, 0, 0, 0, 0);
    for _ in 0..tracks {
        loop {
            let code = *data.get(at).ok_or(super::error::CacheError::Corrupt("music track ends early".into()))?;
            at += 1;
            if code == 7 {
                break;
            }
            if code == 23 {
                tempos += 1;
                continue;
            }
            match code & 0xf {
                0 => note_on += 1,
                1 => note_off += 1,
                2 => controllers += 1,
                3 => bends += 1,
                4 => pressure += 1,
                5 => aftertouch += 1,
                6 => programs += 1,
                _ => return corrupt("unknown music event"),
            }
        }
    }
    let deltas_at = at;
    for _ in 0..tracks + tempos + controllers + note_on + note_off + bends + pressure + aftertouch + programs {
        var_int(data, &mut at)?;
    }
    let controllers_at = at;

    // Pass 2: count controller values per run (bank selects add to the program run).
    let mut sizes = [0usize; 21];
    let mut controller = 0i32;
    for _ in 0..controllers {
        controller = (controller + *data.get(at).ok_or(super::error::CacheError::Corrupt("music track ends early".into()))? as i32) & 0x7f;
        at += 1;
        sizes[controller_run(controller)] += 1;
    }
    sizes[BANK] += programs;
    sizes[AFTERTOUCH_VALUES] = aftertouch;
    sizes[CHANNEL_PRESSURE] = pressure;
    sizes[BEND_HIGH] = bends;
    sizes[BEND_LOW] = bends;
    sizes[NOTES] = note_on + note_off + aftertouch;
    sizes[NOTE_ON_VELOCITY] = note_on;
    sizes[NOTE_OFF_VELOCITY] = note_off;
    sizes[TEMPOS] = tempos * 3;
    let mut runs = Vec::with_capacity(21);
    for size in sizes {
        runs.push(Run { data, at });
        at += size;
    }

    // Pass 3: write the MIDI file.
    let mut out = Vec::new();
    out.extend(b"MThd");
    out.extend(6u32.to_be_bytes());
    out.extend(u16::from(tracks > 1).to_be_bytes());
    out.extend((tracks as u16).to_be_bytes());
    out.extend(division.to_be_bytes());
    let (mut deltas, mut codes, mut controller_numbers) = (deltas_at, 0usize, controllers_at);
    let (mut channel, mut note, mut on_vel, mut off_vel, mut bend, mut press, mut after) = (0i32, 0i32, 0i32, 0i32, 0i32, 0i32, 0i32);
    let mut values = [0i32; 128];
    controller = 0;
    for _ in 0..tracks {
        out.extend(b"MTrk");
        let length_at = out.len();
        out.extend([0; 4]);
        let mut last = -1i32;
        loop {
            put_var(&mut out, var_int(data, &mut deltas)?);
            let code = data[codes] as i32;
            codes += 1;
            let new_status = code != last;
            last = code & 0xf;
            if code == 7 {
                if new_status {
                    out.push(0xff);
                }
                out.extend([0x2f, 0]);
                break;
            }
            if code == 23 {
                if new_status {
                    out.push(0xff);
                }
                out.extend([0x51, 3]);
                for _ in 0..3 {
                    out.push(runs[TEMPOS].next()? as u8);
                }
                continue;
            }
            channel ^= code >> 4;
            let status = |base: i32| (channel + base) as u8;
            match last {
                0 => {
                    if new_status {
                        out.push(status(0x90));
                    }
                    note += runs[NOTES].next()?;
                    on_vel += runs[NOTE_ON_VELOCITY].next()?;
                    out.extend([(note & 0x7f) as u8, (on_vel & 0x7f) as u8]);
                }
                1 => {
                    if new_status {
                        out.push(status(0x80));
                    }
                    note += runs[NOTES].next()?;
                    off_vel += runs[NOTE_OFF_VELOCITY].next()?;
                    out.extend([(note & 0x7f) as u8, (off_vel & 0x7f) as u8]);
                }
                2 => {
                    if new_status {
                        out.push(status(0xb0));
                    }
                    controller = (controller + data[controller_numbers] as i32) & 0x7f;
                    controller_numbers += 1;
                    let value = values[controller as usize] + runs[controller_run(controller)].next()?;
                    values[controller as usize] = value;
                    out.extend([controller as u8, (value & 0x7f) as u8]);
                }
                3 => {
                    if new_status {
                        out.push(status(0xe0));
                    }
                    bend += runs[BEND_LOW].next()?;
                    bend += runs[BEND_HIGH].next()? << 7;
                    out.extend([(bend & 0x7f) as u8, (bend >> 7 & 0x7f) as u8]);
                }
                4 => {
                    if new_status {
                        out.push(status(0xd0));
                    }
                    press += runs[CHANNEL_PRESSURE].next()?;
                    out.push((press & 0x7f) as u8);
                }
                5 => {
                    if new_status {
                        out.push(status(0xa0));
                    }
                    note += runs[NOTES].next()?;
                    after += runs[AFTERTOUCH_VALUES].next()?;
                    out.extend([(note & 0x7f) as u8, (after & 0x7f) as u8]);
                }
                6 => {
                    if new_status {
                        out.push(status(0xc0));
                    }
                    out.push(runs[BANK].next()? as u8);
                }
                _ => return corrupt("unknown music event"),
            }
        }
        let length = (out.len() - length_at - 4) as u32;
        out[length_at..length_at + 4].copy_from_slice(&length.to_be_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuilds_a_one_note_track() {
        // Codes: program change on channel 0, note on, end; deltas 0, 0, 96; one program (5),
        // one note (60) with velocity 100; 1 track, division 96.
        let data = [6, 0, 7, 0, 0, 96, 60, 100, 5, 1, 0, 96];
        let midi = to_midi(&data).unwrap();
        assert_eq!(&midi[..14], &[b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, 0, 1, 0, 96]);
        assert_eq!(&midi[14..18], b"MTrk");
        assert_eq!(&midi[22..], &[0, 0xc0, 5, 0, 0x90, 60, 100, 96, 0xff, 0x2f, 0]);
    }

    #[test]
    fn finds_the_login_track() {
        assert_eq!(login_track(&[1, 0, 9, 1, 0, 3, 0]).unwrap(), Some(9));
    }
}
