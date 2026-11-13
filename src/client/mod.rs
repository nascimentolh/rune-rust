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

mod camera;
pub mod cs2;
mod display;
mod font;
mod gpu;
mod ground;
pub mod interface;
pub mod launch;
pub mod loading_screen;
mod loading_stage;
mod lobby;
mod locations;
mod login;
pub mod login_screen;
pub mod music;
mod object_model;
pub mod palette;
pub mod preferences;
mod raster;
mod scene;
mod shapes;
pub mod source;
mod text_order;
pub mod texture;
mod tiles;
mod ui;
pub mod updater;
pub mod window;
mod world;

use crate::config::{Rsa, Settings};
use loading_screen::LoadingScreens;
use login_screen::{Key, LoginScreen};
use macroquad::prelude::*;
use updater::Summary;
use window::{Screen, Window};
use world::World;

pub use ui::{HEIGHT, WIDTH};

const LOG_OUT: Rect = Rect { x: WIDTH / 2.0 - 70.0, y: 300.0, w: 140.0, h: 36.0 };

pub enum Stage {
    Loading(updater::Shared),
    Login(LoginScreen),
    /// Logged in to the lobby: its window from the cache (needs the login window's `Window`).
    Lobby(crate::protocol::login::LobbyDetails, lobby::LobbyConnection),
    World {
        name: String,
        player_index: u16,
        arrival: login::Arrival,
        /// The game connection, for the packets after `REGION`.
        connection: lobby::LobbyConnection,
    },
}

/// Which screens have been drawn (for the self-test).
#[derive(Default, Debug, Clone, Copy)]
pub struct Drawn {
    pub loading: bool,
    /// A loading screen from the cache (not just the fallback bar) was shown.
    pub cache_loading_screen: bool,
    pub login: bool,
    pub lobby: bool,
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
    /// Log in to the lobby first (`server.cfg` `lobby`).
    lobby: bool,
    /// Show the lobby's top bar (`server.cfg` `lobby_top_bar`).
    lobby_top_bar: bool,
    /// Where the player's preferences are saved; `None` keeps them for this run only.
    pub prefs_file: Option<std::path::PathBuf>,
    /// `Class9.aString99` / `aString102`: the name and password of the last login, used
    /// again for the world login from the lobby.
    credentials: Option<(String, String)>,
    /// A world login started from the lobby (game state 7), until it answers.
    world_login: Option<login::Pending>,
    /// The scene around the player once in the world.
    pub world: Option<World>,
    /// The game camera, around the player.
    camera: camera::Camera,
    /// The colour table's brightness, picked once per run (`CutsceneCameraMovement.method1365`).
    brightness: f64,
}

impl App {
    /// `address` is the server's host:port.
    pub fn new(settings: &Settings, address: String) -> Self {
        // The client holds only the public half of the login key: same modulus, exponent 65537.
        let login_key =
            settings.login_rsa.as_ref().map(|k| Rsa { exponent: crate::protocol::rsa::PUBLIC_EXPONENT.into(), modulus: k.modulus.clone() });
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
            lobby: settings.lobby,
            lobby_top_bar: settings.lobby_top_bar,
            prefs_file: None,
            credentials: None,
            world_login: None,
            world: None,
            camera: camera::Camera::default(),
            brightness: palette::random_brightness(),
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
        login::Target {
            address: self.address.clone(),
            build: self.build,
            sub_build: self.sub_build,
            public_key: self.login_key.clone(),
            lobby: self.lobby,
            display: self.window.as_ref().map_or((1, interface::GAME_WIDTH as u16, interface::GAME_HEIGHT as u16), Window::display),
        }
    }

    /// Where the scene is drawn: the window's game area, or the whole screen without one.
    fn game_area(&self) -> (i32, i32, i32, i32) {
        self.window.as_ref().map_or((0, 0, screen_width() as i32, screen_height() as i32), Window::game_area)
    }

    /// For the self-test: the roof hiding with the player put on the nearest roof tile.
    pub fn roof_check(&mut self) -> Option<String> {
        let area = self.game_area();
        let Stage::World { arrival, .. } = &self.stage else { return None };
        let world = self.world.as_mut()?;
        let plane = arrival.at.2.clamp(0, 3) as usize;
        let focus = world.ground.fine(arrival.at.0, arrival.at.1);
        let view = self.camera.view(&world.ground, focus, area.3, plane);
        let (tile, hidden) = world.roof_check(&view, focus, plane)?;
        Some(format!("standing on roof tile {tile:?} hides {hidden} pieces above"))
    }

    /// Moves between screens when background work finishes.
    pub fn update(&mut self) {
        // The world login Play starts in the lobby (`Class78.method1384` → `worldLogin`).
        let play_target =
            self.window.as_ref().is_some_and(|w| w.ui.play_requested).then(|| login::Target { lobby: false, ..self.login_target() });
        let area = self.game_area();
        match &mut self.stage {
            Stage::Loading(progress) => {
                let (done, assets, session) = {
                    let mut p = progress.lock().unwrap();
                    (p.result.clone(), p.assets.clone(), p.session.take())
                };
                if let Some(session) = session {
                    match Window::open_login(source::Archives::new(Box::new(session)), self.prefs_file.clone()) {
                        Ok(mut w) => {
                            w.ui.game_top_bar = self.lobby_top_bar;
                            self.window = Some(w);
                        }
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
                    w.follow_frame();
                    w.tick();
                    if let Some((name, password)) = w.ui.login_request.take() {
                        self.credentials = Some((name.clone(), password.clone()));
                        let target = login::Target {
                            address: self.address.clone(),
                            build: self.build,
                            sub_build: self.sub_build,
                            public_key: self.login_key.clone(),
                            lobby: self.lobby,
                            display: w.display(),
                        };
                        screen.log_in(name, password, &target);
                    }
                }
                let outcome = screen.poll();
                if let (Some(w), Some(o)) = (&mut self.window, &outcome) {
                    w.ui.login_reply = match o {
                        login::Outcome::Accepted { .. } | login::Outcome::Lobby(..) => 2,
                        login::Outcome::Refused { code, .. } => *code,
                    };
                }
                match outcome {
                    Some(login::Outcome::Accepted { player_index, name, arrival, connection }) => {
                        // Leaving the login screens (`GameState.setGameState`): the login music
                        // fades out and the login window closes until the server opens the game's.
                        if let Some(w) = &mut self.window {
                            w.ui.jukebox.stop();
                            w.ui.close_root();
                            w.ui.in_game = true;
                            self.world = Some(World::build(&mut w.ui.assets, &w.ui.prefs, &arrival, self.brightness));
                        }
                        self.stage = Stage::World { name, player_index, arrival, connection };
                    }
                    Some(login::Outcome::Lobby(details, connection)) => {
                        match self.window.as_mut().map(|w| w.open_lobby(&details, self.lobby_top_bar)) {
                            Some(Ok(())) => self.stage = Stage::Lobby(details, connection),
                            Some(Err(e)) => eprintln!("lobby window unavailable: {e}"),
                            None => eprintln!("lobby needs the login window from the cache"),
                        }
                    }
                    _ => {}
                }
            }
            Stage::Lobby(_, connection) => {
                if let Some(w) = &mut self.window {
                    if let Err(e) = connection.poll(&mut w.ui) {
                        eprintln!("lobby connection: {e}");
                        self.stage = Stage::Login(LoginScreen::new("The lobby connection was lost."));
                        return;
                    }
                    w.follow_frame();
                    w.tick();
                    // `instr6641` → game state 7: log in to the world with the saved name and
                    // password while no login runs (`JS5CacheFile.method3360`); the reply code
                    // goes to the lobby's scripts (`method13784` → `Class9.anInt106`).
                    // `instr6623`: log out to the login screen (`CursorIndexLoader.logout`): the
                    // lobby connection closes (dropped with this stage), the name and password are
                    // forgotten (`BloomPreference.method12708`), and any world login too.
                    if w.ui.logout_requested {
                        self.credentials = None;
                        self.world_login = None;
                        if let Err(e) = w.ui.log_out() {
                            eprintln!("log out: {e}");
                        }
                        self.stage = Stage::Login(LoginScreen::new(""));
                        return;
                    }
                    // The target is worked out before the tick, so a Play this tick starts next frame.
                    if let Some(target) = play_target {
                        w.ui.play_requested = false;
                        if let (None, Some((name, password))) = (&self.world_login, &self.credentials) {
                            w.ui.world_login_reply = -3;
                            self.world_login = Some(login::start(&target, name, password));
                        }
                    }
                    let answer = self.world_login.as_ref().and_then(|p| p.lock().unwrap().take());
                    match answer {
                        Some(login::Outcome::Accepted { player_index, name, arrival, connection }) => {
                            self.world_login = None;
                            w.ui.world_login_reply = 2;
                            // Leaving the lobby (`GameState.setGameState`): the login music fades
                            // out and the lobby's window closes (its connection is dropped).
                            w.ui.jukebox.stop();
                            w.ui.close_root();
                            w.ui.in_game = true;
                            self.world = Some(World::build(&mut w.ui.assets, &w.ui.prefs, &arrival, self.brightness));
                            self.stage = Stage::World { name, player_index, arrival, connection };
                        }
                        Some(login::Outcome::Refused { code, why }) => {
                            self.world_login = None;
                            eprintln!("world login from the lobby refused: {why}");
                            w.ui.world_login_reply = code;
                        }
                        Some(login::Outcome::Lobby(..)) | None => {}
                    }
                }
            }
            Stage::World { arrival, connection, .. } => {
                if let Some(w) = &mut self.window {
                    if let Err(e) = connection.poll(&mut w.ui) {
                        eprintln!("game connection: {e}");
                        if let Err(e) = w.ui.log_out() {
                            eprintln!("log out: {e}");
                        }
                        self.stage = Stage::Login(LoginScreen::new("The connection was lost."));
                        return;
                    }
                    w.follow_frame();
                    w.tick();
                }
                if let Some(world) = &mut self.world {
                    let plane = arrival.at.2.clamp(0, 3) as usize;
                    let focus = world.ground.fine(arrival.at.0, arrival.at.1);
                    self.camera.process(&world.ground, focus, plane);
                    let view = self.camera.view(&world.ground, focus, area.3, plane);
                    world.hide_roofs(&view, focus, plane);
                }
            }
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
        if let (Stage::Login(_) | Stage::Lobby(..), Some(w)) = (&self.stage, &mut self.window) {
            w.read_input();
            return;
        }
        // In the world the game's window gets them too; zoom and log out still work below.
        if let (Stage::World { .. }, Some(w)) = (&self.stage, &mut self.window) {
            w.read_input();
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
            Stage::Lobby(..) => {}
            Stage::World { .. } => {
                // Ctrl + mouse wheel zooms (`Class209_Sub1.mouseWheelMoved`).
                let (_, wheel) = mouse_wheel();
                if wheel != 0.0 && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
                    self.camera.wheel(-(wheel.signum() as i32));
                }
                if click.is_some_and(|at| LOG_OUT.contains(at)) || is_key_pressed(KeyCode::Escape) {
                    // `CursorIndexLoader.logout`: back to the login window, the name and
                    // password forgotten; the game connection closes with this stage.
                    self.credentials = None;
                    if let Some(Err(e)) = self.window.as_mut().map(|w| w.ui.log_out()) {
                        eprintln!("log out: {e}");
                    }
                    self.stage = Stage::Login(LoginScreen::new("You have logged out."));
                }
            }
        }
    }

    pub fn draw(&mut self) {
        let area = self.game_area();
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
                        self.screen.show(&w.raster, w.canvas_x);
                    }
                    None => screen.draw(),
                }
                self.drawn.login = true;
            }
            Stage::Lobby(..) => {
                if let Some(w) = &mut self.window {
                    w.render();
                    self.screen.show(&w.raster, w.canvas_x);
                }
                self.drawn.lobby = true;
            }
            Stage::World { name, player_index, arrival, .. } => {
                match &self.world {
                    Some(world) => {
                        let focus = world.ground.fine(arrival.at.0, arrival.at.1);
                        let view = self.camera.view(&world.ground, focus, area.3, arrival.at.2.clamp(0, 3) as usize);
                        ground::draw(&view, area, &world.scene);
                        // The game's window (`IF_OPENTOP`), laid over the scene.
                        if let Some(w) = self.window.as_mut().filter(|w| w.ui.root_id().is_some()) {
                            w.render_over_scene();
                            self.screen.overlay(&w.raster, w.canvas_x);
                        }
                        ui::button(LOG_OUT, "Log out", true);
                    }
                    None => draw_world_placeholder(name, *player_index, arrival),
                }
                self.drawn.world = true;
            }
        }
    }
}

/// Logged in: the game world isn't built yet, so say so plainly.
fn draw_world_placeholder(name: &str, player_index: u16, arrival: &login::Arrival) {
    ui::background();
    ui::panel(Rect::new(WIDTH / 2.0 - 220.0, 150.0, 440.0, 210.0));
    ui::centred(&format!("Logged in as {name}"), WIDTH / 2.0, 200.0, 28.0, ui::GOOD);
    ui::centred(&format!("Player number {player_index} on this server"), WIDTH / 2.0, 232.0, 18.0, ui::TEXT);
    let (x, y, plane) = arrival.at;
    ui::centred(
        &format!("At {x}, {y} (plane {plane}); map keys for {} regions received", arrival.keys.len()),
        WIDTH / 2.0,
        256.0,
        18.0,
        ui::TEXT,
    );
    ui::centred("The game world isn't drawn yet.", WIDTH / 2.0, 286.0, 18.0, ui::DIM);
    ui::button(LOG_OUT, "Log out", true);
}
