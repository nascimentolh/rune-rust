//! The game client: loading screen while it fetches the cache from the server,
//! then the login screen, then (once logged in) a placeholder for the world.
//!
//! - `updater`        — downloads and checks the cache over JS5, in the reference's stages
//! - `loading_stage`  — the reference's stage table, status text and percentages
//! - `loading_screen` — the reference's loading screens (fallback bar, then index 33 screens)
//! - `raster`, `font` — software drawing and cache fonts, as the reference's Java renderer
//! - `login`          — sends the login (background thread)
//! - `interface`, `cs2` — interfaces (components, layout, drawing) and the script machine
//! - `window`         — the login window (`ENTITY_DEFAULTS.loginWindow`) on screen
//! - `music`          — the MIDI music player (voices, envelopes, mixing)
//! - `preferences`    — the game preferences scripts read
//! - `source`         — cache archives after start-up (the JS5 session, or the local cache)
//! - `login_screen`, `ui` — the plain login form, shown only when the cache has no login
//!   window; it also starts the login the window's script asks for
//! - `launch`         — starts the server process if none is running

pub mod cs2;
mod font;
mod gpu;
pub mod interface;
pub mod launch;
pub mod loading_screen;
mod loading_stage;
mod login;
pub mod login_screen;
pub mod music;
pub mod preferences;
mod raster;
pub mod source;
mod ui;
pub mod updater;
pub mod window;

use crate::config::{Rsa, Settings};
use loading_screen::LoadingScreens;
use login_screen::{Key, LoginScreen};
use macroquad::prelude::*;
use updater::Summary;
use window::{Screen, Window};

pub use ui::{HEIGHT, WIDTH};

const LOG_OUT: Rect = Rect { x: WIDTH / 2.0 - 70.0, y: 300.0, w: 140.0, h: 36.0 };

pub enum Stage {
    Loading(updater::Shared),
    Login(LoginScreen),
    World { name: String, player_index: u16 },
}

/// Which screens have been drawn (for the self-test).
#[derive(Default, Debug, Clone, Copy)]
pub struct Drawn {
    pub loading: bool,
    /// A loading screen from the cache (not just the fallback bar) was shown.
    pub cache_loading_screen: bool,
    pub login: bool,
    pub world: bool,
}

pub struct App {
    pub stage: Stage,
    pub summary: Option<Summary>,
    pub drawn: Drawn,
    pub loading_screens: LoadingScreens,
    /// The login window from the cache, once loading is done.
    pub window: Option<Window>,
    screen: Screen,
    address: String,
    build: i32,
    sub_build: i32,
    grab_key_count: usize,
    js5_key: String,
    login_key: Option<Rsa>,
    /// Where the player's preferences are saved; `None` keeps them for this run only.
    pub prefs_file: Option<std::path::PathBuf>,
}

impl App {
    /// `address` is the server's host:port.
    pub fn new(settings: &Settings, address: String) -> Self {
        // The client holds only the public half of the login key: same modulus, exponent 65537.
        let login_key =
            settings.login_rsa.as_ref().map(|k| Rsa { exponent: crate::login::rsa::PUBLIC_EXPONENT.into(), modulus: k.modulus.clone() });
        let mut app = App {
            stage: Stage::Login(LoginScreen::new("")),
            summary: None,
            drawn: Drawn::default(),
            loading_screens: LoadingScreens::new(),
            window: None,
            screen: Screen::new(),
            address,
            build: settings.build,
            sub_build: settings.sub_build,
            grab_key_count: settings.grab_keys.len(),
            js5_key: settings.js5_key.clone(),
            login_key,
            prefs_file: None,
        };
        app.start_loading();
        app
    }

    fn start_loading(&mut self) {
        let target = updater::Target {
            address: self.address.clone(),
            build: self.build,
            sub_build: self.sub_build,
            js5_key: self.js5_key.clone(),
            grab_key_count: self.grab_key_count,
        };
        self.stage = Stage::Loading(updater::start(target));
    }

    fn login_target(&self) -> login::Target {
        login::Target { address: self.address.clone(), build: self.build, sub_build: self.sub_build, public_key: self.login_key.clone() }
    }

    /// Moves between screens when background work finishes.
    pub fn update(&mut self) {
        match &mut self.stage {
            Stage::Loading(progress) => {
                let (done, assets, session) = {
                    let mut p = progress.lock().unwrap();
                    (p.result.clone(), p.assets.clone(), p.session.take())
                };
                if let Some(session) = session {
                    match Window::open_login(source::Archives::new(Box::new(session)), self.prefs_file.clone()) {
                        Ok(w) => self.window = Some(w),
                        Err(e) => eprintln!("login window unavailable: {e}"),
                    }
                }
                if let Some(assets) = assets {
                    if let Err(e) = self.loading_screens.receive(&assets) {
                        eprintln!("loading screens unavailable, keeping the fallback bar: {e}");
                    }
                }
                if let Some(Ok(summary)) = done {
                    self.summary = Some(summary);
                    self.stage = Stage::Login(LoginScreen::new("Enter your username and password."));
                }
            }
            Stage::Login(screen) => {
                if let Some(w) = &mut self.window {
                    w.tick();
                    if let Some((name, password)) = w.ui.login_request.take() {
                        let target = login::Target {
                            address: self.address.clone(),
                            build: self.build,
                            sub_build: self.sub_build,
                            public_key: self.login_key.clone(),
                        };
                        screen.log_in(name, password, &target);
                    }
                }
                let outcome = screen.poll();
                if let (Some(w), Some(o)) = (&mut self.window, &outcome) {
                    w.ui.login_reply = match o {
                        login::Outcome::Accepted { .. } => 2,
                        login::Outcome::Refused { code, .. } => *code,
                    };
                }
                if let Some(login::Outcome::Accepted { player_index, name }) = outcome {
                    self.stage = Stage::World { name, player_index };
                }
            }
            Stage::World { .. } => {}
        }
    }

    /// A key press (also used by the self-test).
    pub fn key(&mut self, key: Key) {
        let target = self.login_target();
        if let Stage::Login(screen) = &mut self.stage {
            screen.key(key, &target);
        }
    }

    pub fn type_text(&mut self, text: &str) {
        text.chars().for_each(|c| self.key(Key::Char(c)));
    }

    /// Reads the real keyboard and mouse. On the login window they go to its interfaces.
    pub fn handle_input(&mut self) {
        if let (Stage::Login(_), Some(w)) = (&self.stage, &mut self.window) {
            w.read_input();
            return;
        }
        while let Some(c) = get_char_pressed() {
            if !c.is_control() {
                self.key(Key::Char(c));
            }
        }
        for (code, key) in
            [(KeyCode::Backspace, Key::Backspace), (KeyCode::Tab, Key::Tab), (KeyCode::Enter, Key::Enter), (KeyCode::KpEnter, Key::Enter)]
        {
            if is_key_pressed(code) {
                self.key(key);
            }
        }
        let click = is_mouse_button_pressed(MouseButton::Left).then(|| Vec2::from(mouse_position()));
        let target = self.login_target();
        match &mut self.stage {
            Stage::Loading(progress) => {
                let failed = matches!(progress.lock().unwrap().result, Some(Err(_)));
                if failed && is_key_pressed(KeyCode::R) {
                    self.start_loading();
                }
            }
            Stage::Login(screen) => {
                if let Some(at) = click {
                    screen.click(at, &target);
                }
            }
            Stage::World { .. } => {
                if click.is_some_and(|at| LOG_OUT.contains(at)) || is_key_pressed(KeyCode::Escape) {
                    self.stage = Stage::Login(LoginScreen::new("You have logged out."));
                }
            }
        }
    }

    pub fn draw(&mut self) {
        match &self.stage {
            Stage::Loading(progress) => {
                let (status, failure) = {
                    let p = progress.lock().unwrap();
                    (p.status.clone(), p.result.as_ref().and_then(|r| r.as_ref().err().cloned()))
                };
                let drawn = self.loading_screens.draw(&status);
                self.drawn.loading = true;
                self.drawn.cache_loading_screen |= drawn.cache_screen;
                if let Some(e) = failure {
                    // Not the reference's wording: its connection-error screens aren't ported yet.
                    ui::centred(&format!("Error: {e}"), WIDTH / 2.0, HEIGHT - 40.0, 16.0, ui::ERROR);
                    ui::centred("Press R to try again", WIDTH / 2.0, HEIGHT - 20.0, 16.0, ui::DIM);
                }
            }
            Stage::Login(screen) => {
                match &mut self.window {
                    Some(w) => {
                        w.render();
                        self.screen.show(&w.raster);
                    }
                    None => screen.draw(),
                }
                self.drawn.login = true;
            }
            Stage::World { name, player_index } => {
                draw_world_placeholder(name, *player_index);
                self.drawn.world = true;
            }
        }
    }
}

/// Logged in: the game world isn't built yet, so say so plainly.
fn draw_world_placeholder(name: &str, player_index: u16) {
    ui::background();
    ui::panel(Rect::new(WIDTH / 2.0 - 220.0, 150.0, 440.0, 210.0));
    ui::centred(&format!("Logged in as {name}"), WIDTH / 2.0, 200.0, 28.0, ui::GOOD);
    ui::centred(&format!("Player number {player_index} on this server"), WIDTH / 2.0, 232.0, 18.0, ui::TEXT);
    ui::centred("The game world isn't built yet.", WIDTH / 2.0, 266.0, 18.0, ui::DIM);
    ui::button(LOG_OUT, "Log out", true);
}
