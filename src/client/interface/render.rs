//! Drawing interfaces. Port of `IComponentDefinitions.render` (with
//! `RenderAnimIndexLoader.method3629` for sub-interfaces) for the 2D kinds: containers,
//! figures, text, sprites and lines.
//!
//! Not drawn yet: models, rotated sprites (`angle2d`), and the special content types the
//! reference draws with its 3D toolkit (the scene 1337/1403, item views 1338/1339, 1400,
//! the minimap 1401, debug text 1405, 1408 and 1409).

use super::assets::{Assets, Look};
use super::component::Comp;
use super::{Layer, Ui, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::cache::interface::Kind;
use crate::client::font::{HAlign, VAlign};
use crate::client::raster::{Clip, Raster};
use std::collections::HashMap;

const TOOLKIT_CONTENT: [u16; 9] = [1337, 1403, 1338, 1339, 1400, 1401, 1405, 1408, 1409];

pub fn draw(ui: &mut Ui, root: u32, raster: &mut Raster) {
    let Ui { interfaces, subs, assets, .. } = ui;
    let Some(layer) = interfaces.get(&root) else { return };
    let full = Clip { left: 0, top: 0, right: SCREEN_WIDTH, bottom: SCREEN_HEIGHT };
    Painter { interfaces, subs, assets, raster }.layer(layer, None, full, 0, 0);
}

struct Painter<'a> {
    interfaces: &'a HashMap<u32, Layer>,
    subs: &'a HashMap<u32, u32>,
    assets: &'a mut Assets,
    raster: &'a mut Raster,
}

/// The colour with the component's transparency as alpha (`255 - t << 24 | rgb`).
fn with_alpha(transparency: i32, rgb: i32) -> u32 {
    (255 - (transparency as u32 & 0xff)) << 24 | rgb as u32 & 0xff_ffff
}

impl<'a> Painter<'a> {
    /// `render(components, parent, clip, offset)`: every visible child of `parent` in `layer`.
    fn layer(&mut self, layer: &'a Layer, parent: Option<u32>, clip: Clip, ox: i32, oy: i32) {
        self.raster.set_clip(clip.left, clip.top, clip.right, clip.bottom);
        for c in layer.iter().flatten() {
            if c.parent != parent || c.hidden {
                continue;
            }
            let (x, y) = (c.x + ox, c.y + oy);
            let inner = if c.kind == Kind::Other(2) {
                clip
            } else {
                let extra = i32::from(c.kind == Kind::Line);
                Clip {
                    left: x.max(clip.left),
                    top: y.max(clip.top),
                    right: (x + c.width + extra).min(clip.right),
                    bottom: (y + c.height + extra).min(clip.bottom),
                }
            };
            if inner.left >= inner.right || inner.top >= inner.bottom || TOOLKIT_CONTENT.contains(&c.content_type) {
                continue;
            }
            match c.kind {
                Kind::Container => {
                    self.layer(layer, Some(c.id), inner, x - c.scroll_x, y - c.scroll_y);
                    self.layer(&c.children, Some(c.id), inner, x - c.scroll_x, y - c.scroll_y);
                    let interfaces = self.interfaces;
                    if let Some(sub) = self.subs.get(&c.id).and_then(|i| interfaces.get(i)) {
                        self.layer(sub, None, inner, x, y);
                    }
                    self.raster.set_clip(clip.left, clip.top, clip.right, clip.bottom);
                }
                Kind::Figure => self.figure(c, x, y),
                Kind::Text => self.text(c, x, y),
                Kind::Sprite => self.sprite(c, x, y, clip),
                Kind::Line => self.line(c, x, y),
                _ => {}
            }
        }
    }

    fn figure(&mut self, c: &Comp, x: i32, y: i32) {
        // Transparency 0 writes the colour as it is (mode 0); otherwise it blends.
        let argb = if c.transparency == 0 { 0xff00_0000 | c.colour as u32 } else { with_alpha(c.transparency, c.colour) };
        let (w, h) = (c.width, c.height);
        if c.filled {
            self.raster.fill_rect(x, y, w, h, argb);
        } else {
            // `method8430`: the outline inside the w × h box.
            self.raster.fill_rect(x, y, w, 1, argb);
            self.raster.fill_rect(x, y + h - 1, w, 1, argb);
            self.raster.fill_rect(x, y + 1, 1, h - 2, argb);
            self.raster.fill_rect(x + w - 1, y + 1, 1, h - 2, argb);
        }
    }

    fn text(&mut self, c: &Comp, x: i32, y: i32) {
        let Some(font) = self.assets.font(c.font) else { return };
        let shadow = if c.text_shadow { with_alpha(c.transparency, 0) as i32 } else { -1 };
        font.draw_paragraph(
            self.raster,
            &c.text,
            (x, y, c.width, c.height),
            with_alpha(c.transparency, c.colour),
            shadow,
            HAlign::from_code(c.h_align as u8),
            VAlign::from_code(c.v_align as u8),
            c.line_height,
            c.max_lines,
        );
    }

    fn sprite(&mut self, c: &Comp, x: i32, y: i32, clip: Clip) {
        let look = Look { sprite: c.sprite, h_flip: c.h_flip, v_flip: c.v_flip, border: c.border, shadow: c.sprite_shadow };
        let Some(s) = (c.sprite >= 0).then(|| self.assets.sprite(look)).flatten() else { return };
        let plain = c.colour == 0 && c.transparency == 0;
        let tint = (!plain).then(|| with_alpha(c.transparency, if c.colour != 0 { c.colour } else { 0xff_ffff }));
        if c.tiling {
            // `o(...)`: the tiles stay inside the component, then the clip comes back.
            self.raster.set_clip(x.max(clip.left), y.max(clip.top), (x + c.width).min(clip.right), (y + c.height).min(clip.bottom));
            self.raster.draw_sprite_tiled(&s, x, y, c.width, c.height, tint);
            self.raster.set_clip(clip.left, clip.top, clip.right, clip.bottom);
        } else {
            self.raster.draw_sprite_scaled(&s, x, y, c.width, c.height, tint);
        }
    }

    fn line(&mut self, c: &Comp, x: i32, y: i32) {
        let (y1, y2) = if c.line_direction { (y + c.height, y) } else { (y, y + c.height) };
        // ponytail: wider lines (`method8496`) are drawn as parallel one-pixel lines.
        for d in 0..c.line_width.max(1) {
            self.raster.line(x, y1 + d, x + c.width, y2 + d, c.colour as u32 & 0xff_ffff);
        }
    }
}
