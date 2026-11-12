//! The interface on screen: the login window (`ENTITY_DEFAULTS.loginWindow`) opened as the
//! root, drawn each frame by the software raster and shown as one texture.

use super::interface::assets::Assets;
use super::interface::{Ui, SCREEN_HEIGHT, SCREEN_WIDTH};
use super::raster::Raster;
use super::source::Archives;
use macroquad::prelude::*;

pub struct Window {
    pub ui: Ui,
    pub raster: Raster,
}

impl Window {
    /// Opens the login window: lays it out and runs its on-load scripts.
    pub fn open_login(archives: Archives) -> Result<Self, String> {
        let mut assets = Assets::new(archives);
        let id = assets.login_window()?;
        Window::open(assets, id)
    }

    /// Opens any interface as the root window.
    pub fn open(assets: Assets, id: u32) -> Result<Self, String> {
        let mut ui = Ui::new(assets);
        if !ui.load(id) {
            return Err(format!("interface {id} isn't in the cache"));
        }
        ui.open_root(id);
        Ok(Window { ui, raster: Raster::new(SCREEN_WIDTH, SCREEN_HEIGHT) })
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
