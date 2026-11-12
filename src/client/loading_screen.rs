//! The loading screen, as the reference client draws it:
//! 1. until the loading-screen files arrive, the plain fallback bar (`Class158_Sub1_Sub2.method15535`,
//!    used because the client ships none of the applet's bar images);
//! 2. then the cache's loading screens (index 33) in their running order, crossfading
//!    between them (`Class306.run`, `Preference_Sub20.method12808`, `Class299`).

mod elements;

use super::loading_stage::Status;
use super::raster::Raster;
use super::updater::LoadingAssets;
use super::{HEIGHT, WIDTH};
use crate::cache::loading_screens::{Entry, Header, Screen};
use elements::{Assets, BarClock};
use macroquad::prelude::*;
use std::time::Instant;

/// The loading-screen group used by default (`aPreference_Sub11_8217` defaults to 0).
const DEFAULT_GROUP: usize = 0;
/// The fallback colours for colour scheme 0 (`client.aColorArray7186/7199/7200[0]`).
const FALLBACK_FILL: u32 = 0xff8c_1111;
const FALLBACK_OUTLINE: u32 = 0xff8c_1111;
const FALLBACK_TEXT: Color = WHITE;
/// Shown above the fallback bar (`Engine.aString3252`).
const PLEASE_WAIT: &str = "Loading - please wait.";
/// The fallback text font: AWT "Helvetica" bold 13, which Java draws with Arial on Windows.
const FALLBACK_FONT_FILE: &str = "C:/Windows/Fonts/arialbd.ttf";
const FALLBACK_FONT_SIZE: u16 = 13;
/// The loading thread's frame length, which the bar's scroll animation counts in.
const LOADING_FRAME_MS: u128 = 20;

/// What is on screen: the fallback bar or a cache screen (by its place in the order).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Showing {
    Fallback,
    Screen(usize),
}

/// The cache's screens, ready to draw.
struct CacheScreens {
    order: Vec<Entry>,
    screens: Vec<Screen>,
    assets: Assets,
    has_lead_screen: bool,
}

pub struct LoadingScreens {
    cache: Option<CacheScreens>,
    showing: Showing,
    /// `Class306.aLong3601`: when the current screen came up.
    switched_at: Instant,
    /// The screen being faded away from, and its fade time.
    fading_from: Option<(Showing, u32)>,
    /// `Class302.anInt3560`: the last screen switched to, -1 before the first.
    last_switched: i32,
    started: Instant,
    bar_clock: BarClock,
    raster: Raster,
    old_raster: Raster,
    texture: Texture2D,
    fallback_font: Option<Font>,
}

/// What the last frame showed (for the self-test).
#[derive(Clone, Copy, Debug, Default)]
pub struct Drawn {
    pub fallback: bool,
    pub cache_screen: bool,
}

impl LoadingScreens {
    pub fn new() -> Self {
        let fallback_font = std::fs::read(FALLBACK_FONT_FILE).ok().and_then(|bytes| load_ttf_font_from_bytes(&bytes).ok());
        LoadingScreens {
            cache: None,
            showing: Showing::Fallback,
            switched_at: Instant::now(),
            fading_from: None,
            last_switched: -1,
            started: Instant::now(),
            bar_clock: BarClock::default(),
            raster: Raster::new(WIDTH as i32, HEIGHT as i32),
            old_raster: Raster::new(WIDTH as i32, HEIGHT as i32),
            texture: Texture2D::from_rgba8(WIDTH as u16, HEIGHT as u16, &vec![0; (WIDTH * HEIGHT * 4.0) as usize]),
            fallback_font,
        }
    }

    /// Takes the loading-screen files once the updater has them.
    pub fn receive(&mut self, assets: &LoadingAssets) -> Result<(), String> {
        if self.cache.is_some() {
            return Ok(());
        }
        let header_bytes = assets.tips.get(&0).ok_or("no loading screen header (index 33, archive 0)")?;
        let header = Header::decode(header_bytes).map_err(|e| e.to_string())?;
        let order = header.order(DEFAULT_GROUP).to_vec();
        let screens = order
            .iter()
            .map(|e| {
                assets
                    .tips
                    .get(&(e.screen as i32))
                    .ok_or(format!("missing loading screen {}", e.screen))
                    .and_then(|d| Screen::decode(d).map_err(|e| e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let assets = Assets::load(&screens, assets)?;
        self.cache = Some(CacheScreens { order, screens, assets, has_lead_screen: header.has_lead_screen });
        Ok(())
    }

    /// `Class306.method5418`: frames of the loading thread so far.
    fn frames(&self) -> u32 {
        (self.started.elapsed().as_millis() / LOADING_FRAME_MS) as u32
    }

    /// Whether the screen on show has been up long enough (`method189`).
    fn current_done(&self) -> bool {
        match (self.showing, &self.cache) {
            (Showing::Screen(i), Some(c)) => self.switched_at.elapsed().as_millis() >= c.order[i].time_ms as u128,
            _ => true,
        }
    }

    fn fade_time(&self, showing: Showing) -> u32 {
        match (showing, &self.cache) {
            (Showing::Screen(i), Some(c)) => c.order[i].fade_ms,
            _ => 0,
        }
    }

    /// The switching loop of `Preference_Sub20.method12808`.
    fn advance(&mut self) {
        let Some(len) = self.cache.as_ref().map(|c| c.order.len()) else { return };
        let start = (self.last_switched + 1).max(0) as usize;
        for i in start..len {
            if i as i32 - 1 == self.last_switched && self.current_done() {
                self.fading_from = Some((self.showing, self.fade_time(self.showing)));
                self.showing = Showing::Screen(i);
                self.switched_at = Instant::now();
                self.last_switched += 1;
                let has_lead = self.cache.as_ref().is_some_and(|c| c.has_lead_screen);
                if self.last_switched >= len as i32 - 1 && len > 1 {
                    self.last_switched = if has_lead { 0 } else { -1 };
                }
            }
        }
    }

    /// Draws one frame and shows it. Returns what was drawn.
    pub fn draw(&mut self, status: &Status) -> Drawn {
        self.advance();
        let frames = self.frames();
        let mut drawn = Drawn::default();
        // Crossfade (`Class306.run`): old screen at 255 - a over black, then the new one at a.
        let fade = self.fading_from.and_then(|(from, ms)| {
            let elapsed = self.switched_at.elapsed().as_millis() as u32;
            (ms != 0 && elapsed <= ms).then(|| (from, 255 * elapsed / ms))
        });
        if fade.is_none() {
            self.fading_from = None;
        }
        let showing = self.showing;
        drawn.fallback = self.render(showing, status, frames, true);
        drawn.cache_screen = showing != Showing::Fallback;
        if let Some((from, alpha)) = fade {
            std::mem::swap(&mut self.raster, &mut self.old_raster);
            self.render(from, status, frames, false);
            std::mem::swap(&mut self.raster, &mut self.old_raster);
            let new_frame = std::mem::replace(&mut self.raster, Raster::new(WIDTH as i32, HEIGHT as i32));
            self.raster.clear(0);
            self.raster.mix_from(&self.old_raster, 255 - alpha);
            self.raster.mix_from(&new_frame, alpha);
        }
        self.texture.update_from_bytes(WIDTH as u32, HEIGHT as u32, &self.raster.to_rgba());
        clear_background(BLACK);
        draw_texture(&self.texture, 0.0, 0.0, WHITE);
        if drawn.fallback && fade.is_none() {
            self.draw_fallback_text(status);
        }
        drawn
    }

    /// Renders `showing` into the raster. Returns true if it was the fallback.
    fn render(&mut self, showing: Showing, status: &Status, frames: u32, live: bool) -> bool {
        match (showing, &self.cache) {
            (Showing::Screen(i), Some(c)) => {
                // `Class299.method187`: clear to black, then every element in order.
                self.raster.clear(0);
                let clock = if live { Some(&mut self.bar_clock) } else { None };
                elements::draw_screen(&mut self.raster, &c.screens[i], &c.assets, status, frames, clock);
                false
            }
            _ => {
                draw_fallback_bar(&mut self.raster, status.percent);
                true
            }
        }
    }
}

/// The boxes of `method15535`: black screen, outlined box, red fill of 3 px per percent.
fn draw_fallback_bar(r: &mut Raster, percent: i32) {
    r.clear(0);
    let (x, y) = (WIDTH as i32 / 2 - 152, HEIGHT as i32 / 2 - 18);
    let p = percent.clamp(0, 100);
    r.draw_rect(x, y, 303, 33, FALLBACK_OUTLINE);
    r.fill_rect(x + 2, y + 2, p * 3, 30, FALLBACK_FILL);
    r.draw_rect(x + 1, y + 1, 301, 31, 0xff00_0000);
    r.fill_rect(x + p * 3 + 2, y + 2, 300 - p * 3, 30, 0xff00_0000);
}

impl LoadingScreens {
    /// The fallback's two text lines; AWT places text by character count × 6 px.
    fn draw_fallback_text(&self, status: &Status) {
        let (x, y) = (WIDTH as i32 / 2 - 152, HEIGHT as i32 / 2 - 18);
        let params =
            TextParams { font: self.fallback_font.as_ref(), font_size: FALLBACK_FONT_SIZE, color: FALLBACK_TEXT, ..Default::default() };
        let len = status.text.chars().count() as i32;
        draw_text_ex(&status.text, (x + (304 - len * 6) / 2) as f32, (y + 22) as f32, params.clone());
        let wait_len = PLEASE_WAIT.chars().count() as i32;
        draw_text_ex(PLEASE_WAIT, (WIDTH as i32 / 2 - wait_len * 6 / 2) as f32, (HEIGHT as i32 / 2 - 26) as f32, params);
    }

    /// For the self-test: draws screen `n` of the order into a fresh raster and counts
    /// the pixels that aren't black.
    pub fn lit_pixels_of_screen(&mut self, n: usize, status: &Status) -> Option<usize> {
        let cache = self.cache.as_ref()?;
        let screen = cache.screens.get(n)?;
        self.raster.clear(0);
        elements::draw_screen(&mut self.raster, screen, &cache.assets, status, 0, None);
        Some(self.raster.pixels.iter().filter(|&&p| p & 0xff_ffff != 0).count())
    }

    /// Saves what was last drawn into the raster as a PNG (self-test evidence).
    pub fn save_raster_png(&self, path: &std::path::Path) -> Result<(), String> {
        let img = image::RgbaImage::from_raw(self.raster.width as u32, self.raster.height as u32, self.raster.to_rgba())
            .ok_or("raster size mismatch")?;
        img.save(path).map_err(|e| e.to_string())
    }

    /// Screens in the running order, once the files have arrived.
    pub fn screen_count(&self) -> usize {
        self.cache.as_ref().map_or(0, |c| c.order.len())
    }
}

/// A mid-load status, for drawing screens outside a real load (self-test).
pub fn sample_status() -> Status {
    let mut s = Status::new();
    s.update(6, 0);
    s.update(6, 50);
    s
}
