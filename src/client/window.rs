//! The interface on screen: the login window (`ENTITY_DEFAULTS.loginWindow`) opened as the
//! root, drawn each frame by the software raster and shown as one texture.

use super::interface::assets::Assets;
use super::interface::input::Input;
use super::interface::{Ui, SCREEN_HEIGHT, SCREEN_WIDTH};
use super::raster::Raster;
use super::source::Archives;
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
        Ok(Window { ui, raster: Raster::new(SCREEN_WIDTH, SCREEN_HEIGHT), input: Input::default(), ticked })
    }

    /// Reads the mouse and keyboard (`CanvasKeyRecorder`, the mouse recorder). Presses come
    /// first with char 65535, then typed characters with code -1, as `method3235` orders them.
    pub fn read_input(&mut self) {
        let (mx, my) = mouse_position();
        self.input.mouse = (mx as i32, my as i32);
        self.input.button_down = is_mouse_button_down(MouseButton::Left);
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
                Input { mouse: self.input.mouse, button_down: self.input.button_down, ..Default::default() }
            };
            self.input.mouse = input.mouse;
            self.input.button_down = input.button_down;
            self.ui.tick(&input);
        }
    }

    /// Draws the interfaces into the raster (over black: the 3D scene isn't drawn yet).
    pub fn render(&mut self) {
        self.raster.clear(0);
        self.ui.draw(&mut self.raster);
    }
}

/// Shows a raster as the whole client area.
pub struct Screen {
    texture: Texture2D,
}

impl Screen {
    pub fn new() -> Self {
        let texture =
            Texture2D::from_rgba8(SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16, &vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize]);
        texture.set_filter(FilterMode::Nearest);
        Screen { texture }
    }

    pub fn show(&self, raster: &Raster) {
        self.texture.update_from_bytes(raster.width as u32, raster.height as u32, &raster.to_rgba());
        clear_background(BLACK);
        draw_texture(&self.texture, 0.0, 0.0, WHITE);
    }
}
