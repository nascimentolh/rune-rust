//! Which song plays, its volume and fades. Port of `Class148` (`anInt1730`: 0 idle,
//! 1 fading out, 2 loading the next song), `Class339.method6047` / `Class109.method1859`
//! (start), `Class358.method6240` (stop), `ParticleEmitterConfig.method1491` (volume),
//! `VarBitDefinitions.method3805` (cut), `MaterialProp33.method15420` (every client tick)
//! and `ProcessorSpecs.method7730` (load, then play).
//!
//! The reference mixes the song live; here a background thread renders the whole song once
//! at the volume it starts with (`load::Raw::render`; the volume decides what clips) and
//! macroquad plays it. Later volume changes (fades, the slider) scale the sound by
//! new / rendered volume: `setVolume` scales every voice linearly (`v × volume >> 8`).
//! State 3 (fade in by `Class383.anInt4664`) is left out: that step is only ever set to 0.

use super::load::{self, Raw};
use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};
use std::future::Future;
use std::sync::mpsc::{channel, Receiver};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

/// A rendered song: its sound file and length, or why it failed.
type Rendered = Result<(Vec<u8>, Duration), String>;

/// Songs longer than this are cut (the login track is a few minutes).
const MAX_SECONDS: usize = 600;

/// `Node_Sub15_Sub2.volume` starts at 256.
const FULL: i32 = 256;

struct Playing {
    sound: Sound,
    /// The player volume the song was rendered at.
    rendered_at: i32,
    started: Instant,
    length: Duration,
}

pub struct Jukebox {
    /// False for the tools and the self-test, which have no sound device.
    pub audio: bool,
    /// `Class148.anInt1730`.
    state: u8,
    /// `Friend.anInt46` (with `CutsceneEntity.aClass317_746`): the song to load next.
    pending: Option<u32>,
    /// `Class152.aBool1962`: the next song loops.
    looping: bool,
    /// `Class158_Sub2_Sub3.anInt10243`: its volume.
    target: i32,
    /// `CutsceneAction_Sub22.anInt9440`: volume taken off per tick while fading out.
    step: i32,
    /// The player's volume (`method15123` / `setVolume`).
    volume: i32,
    playing: Option<Playing>,
    /// The song being rendered in the background: its sound file and length.
    loading: Option<Receiver<Rendered>>,
    /// The volume the song loading is rendered at.
    rendering_at: i32,
    /// `Class260.anInt3223`: the song the client means to play (-1 none).
    pub song: i32,
}

impl Default for Jukebox {
    fn default() -> Self {
        Jukebox {
            audio: false,
            state: 0,
            pending: None,
            looping: false,
            target: 0,
            step: 2,
            volume: FULL,
            playing: None,
            loading: None,
            rendering_at: 0,
            song: -1,
        }
    }
}

/// Runs a future macroquad finishes at once on the desktop (`load_sound_from_bytes`).
fn now<F: Future>(f: F) -> Option<F::Output> {
    let mut f = std::pin::pin!(f);
    match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => Some(v),
        Poll::Pending => None,
    }
}

impl Jukebox {
    /// `Class339.method6047` (step 2: fades the old song out) and `Class109.method1859`
    /// (step 10000: cuts it), for song `track` at `volume`.
    pub fn start(&mut self, track: u32, volume: i32, step: i32) {
        self.state = 1;
        self.pending = Some(track);
        self.target = volume;
        self.looping = false;
        self.step = step;
    }

    /// `Class358.method6240`: fade out, nothing next.
    pub fn stop(&mut self) {
        self.state = 1;
        self.pending = None;
        self.target = 0;
        self.looping = false;
        self.step = 2;
    }

    /// `VarBitDefinitions.method3805`: stop at once, nothing next.
    pub fn cut(&mut self) {
        self.halt();
        self.state = 1;
        self.pending = None;
    }

    /// `ParticleEmitterConfig.method1491`: the volume of the song coming, else of this one.
    pub fn set_volume(&mut self, v: i32) {
        if self.state != 0 {
            self.target = v;
        } else {
            self.apply_volume(v);
        }
    }

    /// What the player is doing, for the self-test.
    pub fn status(&self) -> &'static str {
        match (self.is_playing(), self.loading.is_some(), self.state) {
            (true, ..) => "playing",
            (_, true, _) => "loading",
            (_, _, 0) => "idle",
            _ => "fading or waiting",
        }
    }

    /// `method15103`: a song is playing.
    pub fn is_playing(&self) -> bool {
        self.playing.as_ref().is_some_and(|p| p.started.elapsed() < p.length)
    }

    fn apply_volume(&mut self, v: i32) {
        self.volume = v;
        if let Some(p) = &self.playing {
            let scale = if p.rendered_at > 0 { v.max(0) as f32 / p.rendered_at as f32 } else { 0.0 };
            set_sound_volume(&p.sound, scale);
        }
    }

    /// `method15101`: stops the song.
    fn halt(&mut self) {
        if let Some(p) = self.playing.take() {
            stop_sound(&p.sound);
        }
    }

    /// One client tick: `method15420`, then `method7730`. `raw` gathers a song's data when one
    /// is due (where the cache is).
    pub fn tick(&mut self, stereo: bool, raw: &mut dyn FnMut(u32) -> Result<Raw, String>) {
        if !self.audio {
            return;
        }
        if self.state == 1 {
            let v = self.volume;
            if v > 0 && self.is_playing() {
                self.apply_volume((v - self.step).max(0));
                return;
            }
            self.halt();
            // `Node_Sub44_Sub3.aNode_Sub7_9563 = null`: a song half loaded is dropped.
            self.loading = None;
            self.state = if self.pending.is_some() { 2 } else { 0 };
        }
        if self.state != 2 {
            return;
        }
        let Some(track) = self.pending else { return };
        let Some(rx) = &self.loading else {
            match raw(track) {
                Ok(song) => {
                    let (tx, rx) = channel();
                    let volume = self.target;
                    self.rendering_at = volume;
                    std::thread::spawn(move || {
                        let result = song.render(stereo, volume, MAX_SECONDS).map(|pcm| {
                            let frames = pcm.len() / if stereo { 2 } else { 1 };
                            (load::wav(&pcm, stereo), Duration::from_secs_f64(frames as f64 / super::RATE as f64))
                        });
                        let _ = tx.send(result);
                    });
                    self.loading = Some(rx);
                }
                Err(e) => {
                    eprintln!("music {track}: {e}");
                    (self.state, self.pending) = (0, None);
                }
            }
            return;
        };
        let Ok(result) = rx.try_recv() else { return };
        self.loading = None;
        (self.state, self.pending) = (0, None);
        match result.and_then(|(wav, length)| {
            let sound = now(load_sound_from_bytes(&wav)).ok_or("the sound didn't load")?.map_err(|e| e.to_string())?;
            Ok((sound, length))
        }) {
            Ok((sound, length)) => {
                play_sound(&sound, PlaySoundParams { looped: self.looping, volume: 0.0 });
                self.playing = Some(Playing { sound, rendered_at: self.rendering_at, started: Instant::now(), length });
                self.apply_volume(self.target);
            }
            Err(e) => eprintln!("music {track}: {e}"),
        }
    }
}
