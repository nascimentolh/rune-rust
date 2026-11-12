//! Getting a song ready to play: its patches and the samples their notes use. Port of
//! `Class250` / `Node_Sub14.method12216` (only the notes the song plays,
//! `Node_Sub7.method12165`) with `MIDIInstrument` (index 14, Vorbis) and `SoundEffect`
//! (index 4, synthesized) samples.
//!
//! Split in two like the reference, which decodes samples a little at a time in the
//! background: `gather` fetches the archives (cheap, where the cache lives), `decode` turns
//! them into playable samples (slow, anywhere: everything in `Raw` can move to a thread).

use super::voice::SampleData;
use super::LoadedPatch;
use crate::cache::{midi, music_patch::Patch, sound_effect, vorbis};
use crate::client::source::Files;
use std::collections::HashMap;
use std::rc::Rc;

/// Each note a patch plays, with the sample kind it uses.
type NoteSamples = Vec<(usize, i32)>;

/// A song with the raw data of everything it plays.
pub struct Raw {
    pub midi: Vec<u8>,
    /// The Vorbis setup, file 0/0 of index 14.
    setup: Vec<u8>,
    /// Program → patch, with the sample kind each note uses (only notes the song plays).
    patches: Vec<(i32, Patch, NoteSamples)>,
    /// Sample kind → its file: odd kinds Vorbis (index 14), even ones synthesized (index 4).
    samples: HashMap<i32, Vec<u8>>,
}

fn first(files: Option<Rc<Files>>, file: usize) -> Vec<u8> {
    files.and_then(|f| f.get(file).cloned().flatten()).unwrap_or_default()
}

/// The archives the song needs. `fetch` gives an archive's files; `single_synth` says index 4
/// has one archive, so sound n is its file n (`Index.getFile`), else archive n's file 0.
pub fn gather(fetch: &mut dyn FnMut(u8, u32) -> Option<Rc<Files>>, single_synth: bool, midi: Vec<u8>) -> Result<Raw, String> {
    let setup = first(fetch(14, 0), 0);
    let mut patches = Vec::new();
    let mut samples = HashMap::new();
    for (program, notes) in midi::used_notes(&midi) {
        let Some(files) = fetch(15, program as u32) else { continue };
        let patch =
            Patch::decode(files.first().cloned().flatten().as_deref().unwrap_or(&[])).map_err(|e| format!("patch {program}: {e}"))?;
        let mut keys = Vec::new();
        for (key, &raw) in patch.samples.iter().enumerate() {
            if !notes[key] || raw == 0 {
                continue;
            }
            let kind = raw - 1;
            keys.push((key, kind));
            if samples.contains_key(&kind) {
                continue;
            }
            let id = (kind >> 2) as u32;
            let bytes = if kind & 1 == 1 {
                first(fetch(14, id), 0)
            } else if single_synth {
                first(fetch(4, 0), id as usize)
            } else {
                first(fetch(4, id), 0)
            };
            samples.insert(kind, bytes);
        }
        patches.push((program, patch, keys));
    }
    Ok(Raw { midi, setup, patches, samples })
}

impl Raw {
    /// The patches with their samples decoded (8-bit synthesized samples mix as b / 128).
    pub fn decode(&self) -> Result<HashMap<i32, LoadedPatch>, String> {
        let setup = vorbis::Setup::decode(&self.setup).map_err(|e| format!("vorbis setup: {e}"))?;
        let mut decoded: HashMap<i32, Rc<SampleData>> = HashMap::new();
        for (&kind, bytes) in &self.samples {
            let data = if kind & 1 == 1 {
                let v = vorbis::decode_sample(&setup, bytes).map_err(|e| format!("vorbis {}: {e}", kind >> 2))?;
                SampleData { rate: v.rate, pcm: v.pcm, loop_start: v.loop_start, loop_end: v.loop_end, ping_pong: v.loop_flag }
            } else {
                let s = sound_effect::synthesize(bytes).map_err(|e| format!("synth {}: {e}", kind >> 2))?;
                SampleData {
                    rate: 22050,
                    pcm: s.pcm.iter().map(|&b| b as f32 / 128.0).collect(),
                    loop_start: s.loop_start,
                    loop_end: s.loop_end,
                    ping_pong: false,
                }
            };
            decoded.insert(kind, Rc::new(data));
        }
        Ok(self
            .patches
            .iter()
            .map(|(program, patch, keys)| {
                let mut samples = vec![None; 128];
                for &(key, kind) in keys {
                    samples[key] = decoded.get(&kind).cloned();
                }
                (*program, LoadedPatch { patch: patch.clone(), samples })
            })
            .collect())
    }

    /// The whole song played once at player volume `volume` (`setVolume`, applied before the
    /// mix is clamped, so it decides what clips), as 16-bit PCM at 22,050 Hz, rendered 256
    /// frames at a time like `Class253.method4337`; at most `max_seconds`.
    pub fn render(&self, stereo: bool, volume: i32, max_seconds: usize) -> Result<Vec<i16>, String> {
        let mut player = super::Player::new(stereo);
        player.set_volume(volume);
        player.patches = self.decode()?;
        player.play(&self.midi, false);
        let channels = if stereo { 2 } else { 1 };
        let mut pcm = Vec::new();
        let mut block = vec![0i32; 256 * channels];
        for _ in 0..(super::RATE as usize * max_seconds).div_ceil(256) {
            if !player.playing() {
                break;
            }
            block.fill(0);
            player.render(&mut block, 0, 256);
            pcm.extend(super::to_pcm16(&block));
        }
        Ok(pcm)
    }
}

/// A 16-bit PCM WAV file at 22,050 Hz.
pub fn wav(pcm: &[i16], stereo: bool) -> Vec<u8> {
    let channels: u16 = if stereo { 2 } else { 1 };
    let rate = super::RATE as u32;
    let data_len = (pcm.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + pcm.len() * 2);
    out.extend(b"RIFF");
    out.extend((36 + data_len).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(channels.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * 2 * channels as u32).to_le_bytes());
    out.extend((2 * channels).to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend(data_len.to_le_bytes());
    for v in pcm {
        out.extend(v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_says_what_follows() {
        let w = wav(&[1, -1, 2, -2], true);
        assert_eq!((&w[..4], &w[8..12], w.len()), (&b"RIFF"[..], &b"WAVE"[..], 44 + 8));
        assert_eq!(u16::from_le_bytes([w[22], w[23]]), 2);
    }
}
