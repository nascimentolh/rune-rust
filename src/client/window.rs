//! The interface on screen: the login window (`ENTITY_DEFAULTS.loginWindow`) opened as the
//! root, drawn each frame by the software raster and shown as one texture.
//!
//! The window can be resized and maximised like the reference's frame
//! (`engineFrame.setResizable(true)`). The game area inside it follows the screen size mode
//! (`UID192.method7373` → `Class507.method8725`): mode 1 is the fixed 765 × 503, mode 2 the
//! window's size, at most 800 × 600 or 1024 × 768 by the "max screen size" preference
//! (`Class46.method935`); either way centred across the top. A new window size is applied
//! 500 ms after it last changed (`client.aLong7307`). Full screen (mode 3) isn't ported: it
//! falls back to the saved mode, as the reference does when it can't go full screen.

use super::interface::assets::Assets;
use super::interface::input::Input;
use super::interface::{Ui, GAME_HEIGHT, GAME_WIDTH};
use super::raster::Raster;
use super::source::Archives;
use crate::protocol::client_prot;
use macroquad::prelude::*;

/// `CanvasKeyRecorder.anIntArray8044`: Java key code (`KeyEvent.VK_*`) → client key code
/// (0 = ignored). Codes past the end are 0.
const KEY_CODES: [i32; 156] = [
    0, 0, 0, 0, 0, 0, 0, 0, 85, 80, 84, 0, 91, 0, 0, 0, 81, 82, 86, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 83, 104, 105, 103, 102, 96, 98,
    97, 99, 0, 0, 0, 0, 0, 0, 0, 25, 16, 17, 18, 19, 20, 21, 22, 23, 24, 0, 0, 0, 0, 0, 0, 0, 48, 68, 66, 50, 34, 51, 52, 53, 39, 54, 55,
    56, 70, 69, 40, 41, 32, 35, 49, 36, 38, 67, 33, 65, 37, 64, 0, 0, 0, 0, 0, 228, 231, 227, 233, 224, 219, 225, 230, 226, 232, 89, 87, 0,
    88, 229, 90, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0, 0, 0, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 100,
];

pub struct Window {
    pub ui: Ui,
    pub raster: Raster,
    /// Input gathered since the last tick.
    input: Input,
    /// The last tick run, in `CYCLES_20MS`.
    ticked: i32,
    /// `SunIndexLoader.anInt434` × `Class107.anInt1082`: the window's inside size.
    frame: (i32, i32),
    /// `client.GAME_CANVAS_X`: where the game area starts across the window.
    pub canvas_x: i32,
    /// `client.aLong7307`: when to apply a new window size.
    resize_at: Option<std::time::Instant>,
}

/// The window's inside size now.
fn frame_size() -> (i32, i32) {
    ((screen_width() as i32).max(1), (screen_height() as i32).max(1))
}

/// `Class46.method935`: the game area in screen size mode 2, by "max screen size" (2: up to
/// 800 × 600, 1: up to 1024 × 768, 0: the whole window).
fn resizable_area(frame: (i32, i32), max_screen_size: i32) -> (i32, i32) {
    match max_screen_size {
        2 => (frame.0.min(800), frame.1.min(600)),
        1 => (frame.0.min(1024), frame.1.min(768)),
        _ => frame,
    }
}

/// The Java `VK_*` code of a key (what `KeyEvent.getKeyCode` gives).
fn java_key_code(k: KeyCode) -> Option<usize> {
    use KeyCode::*;
    let letters = [A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z];
    let digits = [Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9];
    let functions = [F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12];
    if let Some(i) = letters.iter().position(|&l| l == k) {
        return Some(65 + i);
    }
    if let Some(i) = digits.iter().position(|&d| d == k) {
        return Some(48 + i);
    }
    if let Some(i) = functions.iter().position(|&f| f == k) {
        return Some(112 + i);
    }
    Some(match k {
        Backspace => 8,
        Tab => 9,
        Enter | KpEnter => 10,
        LeftShift | RightShift => 16,
        LeftControl | RightControl => 17,
        LeftAlt | RightAlt => 18,
        Escape => 27,
        Space => 32,
        PageUp => 33,
        PageDown => 34,
        End => 35,
        Home => 36,
        Left => 37,
        Up => 38,
        Right => 39,
        Down => 40,
        Delete => 127,
        Insert => 155,
        _ => return None,
    })
}

/// The character AWT's `keyTyped` gives for keys that type no printable character.
fn typed_control(k: KeyCode) -> Option<i32> {
    Some(match k {
        KeyCode::Backspace => 8,
        KeyCode::Tab => 9,
        KeyCode::Enter | KeyCode::KpEnter => 10,
        KeyCode::Escape => 27,
        KeyCode::Delete => 127,
        _ => return None,
    })
}

/// `Class175.method2962`: the characters the client accepts as typed.
fn accepted(c: char) -> bool {
    let v = c as u32;
    (1..128).contains(&v) || (160..=255).contains(&v) || crate::cache::text::char_to_cp1252(c) >= 128
}

impl Window {
    /// Opens the login window: lays it out and runs its on-load scripts, with the
    /// preferences saved in `prefs_file` (the defaults when `None`).
    pub fn open_login(archives: Archives, prefs_file: Option<std::path::PathBuf>) -> Result<Self, String> {
        let mut assets = Assets::new(archives);
        let id = assets.login_window()?;
        let mut w = Window::open(assets, id, prefs_file)?;
        w.ui.can_profile_gpu = true;
        w.ui.jukebox.audio = true;
        // `IndexLoaders`: the saved screen size mode, once loading is done.
        w.frame = frame_size();
        let mode = w.ui.prefs.screen_size;
        w.set_mode(mode);
        w.ui.enter_login_screen();
        Ok(w)
    }

    /// After a lobby login: the lobby window becomes the root (`Ui::open_lobby`).
    pub fn open_lobby(&mut self, details: &crate::protocol::login::LobbyDetails, top_bar: bool) -> Result<(), String> {
        self.ui.open_lobby(details, top_bar)
    }

    /// Opens any interface as the root window.
    pub fn open(assets: Assets, id: u32, prefs_file: Option<std::path::PathBuf>) -> Result<Self, String> {
        let mut ui = Ui::new(assets);
        if let Some(path) = prefs_file {
            ui.use_saved_prefs(path);
        }
        if !ui.load(id) {
            return Err(format!("interface {id} isn't in the cache"));
        }
        ui.open_root(id);
        let ticked = ui.clock();
        Ok(Window {
            ui,
            raster: Raster::new(GAME_WIDTH, GAME_HEIGHT),
            input: Input::default(),
            ticked,
            frame: (GAME_WIDTH, GAME_HEIGHT),
            canvas_x: 0,
            resize_at: None,
        })
    }

    /// `Class507.method8725` for mode 1 or 2: the game area's size and place in the window,
    /// then the root interface laid out again with resize hooks (`Class516.method8867(true)`),
    /// and in the game the server told (`Class388.method6692`: `SCREEN_SIZE`).
    pub fn set_mode(&mut self, mode: i32) {
        let fw = self.frame.0;
        let (w, h) = if mode != 1 { resizable_area(self.frame, self.ui.prefs.max_screen_size) } else { (GAME_WIDTH, GAME_HEIGHT) };
        self.canvas_x = (fw - w) / 2;
        self.ui.resizable = mode >= 2;
        if (w, h) != (self.ui.width, self.ui.height) {
            (self.ui.width, self.ui.height) = (w, h);
            self.ui.layout_after_resize();
        }
        if self.ui.in_game {
            let (mode, width, height) = self.display();
            let packet = client_prot::screen_size(mode, width, height, self.ui.prefs.anti_aliasing as u8);
            self.ui.outgoing.push((client_prot::SCREEN_SIZE, packet));
        }
    }

    /// `Class158.windowedMode` and the game area's size, as the login and `SCREEN_SIZE` send
    /// them. (No full screen mode 3 here: the window is maximised instead.)
    pub fn display(&self) -> (u8, u16, u16) {
        (if self.ui.resizable { 2 } else { 1 }, self.ui.width as u16, self.ui.height as u16)
    }

    /// The game area in the window: x, y, width, height (`GAME_CANVAS_X`, `GAME_CANVAS_Y` 0,
    /// `BASE_WINDOW_WIDTH`, `BASE_WINDOW_HEIGHT`). The scene is drawn only inside it.
    pub fn game_area(&self) -> (i32, i32, i32, i32) {
        (self.canvas_x, 0, self.ui.width, self.ui.height)
    }

    /// `client.method11620`'s window check, each frame: a changed window size is applied
    /// 500 ms later in the current mode (`Class158.windowedMode`); a mode a script asked for
    /// (`instr6645`) at once.
    pub fn follow_frame(&mut self) {
        let now = frame_size();
        if now != self.frame {
            self.frame = now;
            self.resize_at = Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
        }
        if self.resize_at.is_some_and(|at| std::time::Instant::now() > at) {
            self.resize_at = None;
            let mode = if self.ui.resizable { 2 } else { 1 };
            self.set_mode(mode);
        }
        if let Some(mode) = self.ui.mode_request.take() {
            self.set_mode(mode);
        }
    }

    /// Reads the mouse and keyboard (`CanvasKeyRecorder`, the mouse recorder). Presses come
    /// first with char 65535, then typed characters with code -1, as `method3235` orders them.
    pub fn read_input(&mut self) {
        let (mx, my) = mouse_position();
        self.input.mouse = (mx as i32 - self.canvas_x, my as i32);
        self.input.button_down = is_mouse_button_down(MouseButton::Left);
        self.input.other_buttons_down = [is_mouse_button_down(MouseButton::Right), is_mouse_button_down(MouseButton::Middle)];
        if is_mouse_button_pressed(MouseButton::Left) && self.input.click.is_none() {
            self.input.click = Some(self.input.mouse);
        }
        // `mouseWheelMoved`: one notch each, positive towards the user (macroquad's y is the
        // other way round).
        let (_, wheel) = mouse_wheel();
        if wheel != 0.0 {
            self.input.wheel -= wheel.signum() as i32;
        }
        let mut typed = Vec::new();
        for k in get_keys_pressed() {
            // `method13378` on a press: codes with bit 0x80 are dropped.
            if let Some(&code) = java_key_code(k).and_then(|vk| KEY_CODES.get(vk)).filter(|&&c| c != 0 && c & 0x80 == 0) {
                self.input.keys.push((code, 65535));
            }
            typed.extend(typed_control(k));
        }
        while let Some(c) = get_char_pressed() {
            if !c.is_control() && accepted(c) {
                typed.push(c as i32);
            }
        }
        self.input.keys.extend(typed.into_iter().map(|c| (-1, c)));
    }

    /// Runs the client ticks due since the last frame; the gathered input goes to the first.
    pub fn tick(&mut self) {
        let due = (self.ui.clock() - self.ticked).clamp(0, 5);
        self.ticked = self.ui.clock();
        for n in 0..due {
            let input = if n == 0 {
                std::mem::take(&mut self.input)
            } else {
                Input {
                    mouse: self.input.mouse,
                    button_down: self.input.button_down,
                    other_buttons_down: self.input.other_buttons_down,
                    ..Default::default()
                }
            };
            self.input.mouse = input.mouse;
            self.input.button_down = input.button_down;
            self.input.other_buttons_down = input.other_buttons_down;
            self.ui.tick(&input);
        }
    }

    /// Draws the interfaces into the raster, over black.
    pub fn render(&mut self) {
        self.fit_raster();
        self.raster.clear(0);
        self.ui.draw(&mut self.raster);
    }

    /// Draws the interfaces into the raster with gaps where nothing is drawn, to lay over the
    /// 3D scene (the reference draws both into one buffer, the scene first).
    pub fn render_over_scene(&mut self) {
        self.fit_raster();
        self.raster.clear_see_through();
        self.ui.draw(&mut self.raster);
    }

    fn fit_raster(&mut self) {
        if (self.raster.width, self.raster.height) != (self.ui.width, self.ui.height) {
            self.raster = Raster::new(self.ui.width, self.ui.height);
        }
    }
}

/// Shows a raster as the game area: at `x` across the top of the window, black around it.
pub struct Screen {
    texture: Option<Texture2D>,
}

impl Screen {
    pub fn new() -> Self {
        Screen { texture: None }
    }

    pub fn show(&mut self, raster: &Raster, x: i32) {
        clear_background(BLACK);
        self.overlay(raster, x);
    }

    /// Draws the raster over what is already on screen; its gaps let that show.
    pub fn overlay(&mut self, raster: &Raster, x: i32) {
        let size = (raster.width as f32, raster.height as f32);
        let texture = match &self.texture {
            Some(t) if (t.width(), t.height()) == size => t,
            _ => {
                let t = Texture2D::from_rgba8(raster.width as u16, raster.height as u16, &raster.to_rgba());
                t.set_filter(FilterMode::Nearest);
                self.texture.insert(t)
            }
        };
        texture.update_from_bytes(raster.width as u32, raster.height as u32, &raster.to_rgba());
        draw_texture(texture, x as f32, 0.0, WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resizable_area_is_capped_by_max_screen_size() {
        assert_eq!(resizable_area((1920, 1080), 0), (1920, 1080));
        assert_eq!(resizable_area((1920, 1080), 1), (1024, 768));
        assert_eq!(resizable_area((1920, 1080), 2), (800, 600));
        assert_eq!(resizable_area((700, 500), 1), (700, 500));
    }
}
