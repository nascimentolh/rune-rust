//! Drawing loading-screen elements. Ports of `Class42` (image), `Class51` (text) and
//! `Class52` / `Class52_Sub2` / `Class52_Sub2_Sub1` (image-built progress bar).

use crate::cache::font_metrics::FontMetrics;
use crate::cache::loading_screens::{BarBox, BarImages, Element, Screen};
use crate::cache::sprite::{self, Sprite};
use crate::client::font::{Font, HAlign, VAlign};
use crate::client::loading_stage::Status;
use crate::client::raster::Raster;
use crate::client::updater::LoadingAssets;
use crate::client::{HEIGHT, WIDTH};
use std::collections::HashMap;
use std::time::Instant;

/// Status text gets " (stage)" after the bar hasn't moved for this long.
const STALL_MS: u128 = 10000;
/// Vertical offset of the bar's text below its centre (`method20`).
const BAR_TEXT_DROP: i32 = 4;

/// Decoded images (index 32) and fonts (index 32 glyphs + index 13 metrics) by id.
pub struct Assets {
    images: HashMap<i32, Sprite>,
    fonts: HashMap<i32, Font>,
}

impl Assets {
    pub fn load(screens: &[Screen], raw: &LoadingAssets) -> Result<Self, String> {
        let mut images = HashMap::new();
        for id in screens.iter().flat_map(Screen::images) {
            if images.contains_key(&id) {
                continue;
            }
            let bytes = raw.images.get(&id).ok_or(format!("missing loading image {id}"))?;
            images.insert(id, decode_image(bytes).map_err(|e| format!("image {id}: {e}"))?);
        }
        let mut fonts = HashMap::new();
        for id in screens.iter().flat_map(Screen::fonts) {
            if fonts.contains_key(&id) {
                continue;
            }
            let glyphs = raw.images.get(&id).ok_or(format!("missing font glyphs {id}"))?;
            let metrics = raw.metrics.get(&id).ok_or(format!("missing font metrics {id}"))?;
            let font =
                Font::new(FontMetrics::decode(metrics).map_err(|e| e.to_string())?, sprite::decode(glyphs).map_err(|e| e.to_string())?);
            fonts.insert(id, font);
        }
        Ok(Assets { images, fonts })
    }
}

/// `Class103_Sub1.method14490`: loading-screen images are ordinary picture files (JPEG here),
/// because the client's image test succeeds and it reads index 32, not 34.
fn decode_image(bytes: &[u8]) -> Result<Sprite, String> {
    if let Some(sprite) = decode_ycca_jpeg(bytes)? {
        return Ok(sprite);
    }
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?.to_rgba8();
    let (w, h) = img.dimensions();
    let pixels = img.pixels().map(|p| (p[3] as u32) << 24 | (p[0] as u32) << 16 | (p[1] as u32) << 8 | p[2] as u32).collect();
    Ok(Sprite { min_x: 0, min_y: 0, width: w as usize, height: h as usize, pixels })
}

/// The loading images are 4-channel JPEGs with no Adobe marker. Java's decoder reads those as
/// YCbCr plus alpha; most decoders guess CMYK instead. Returns `None` for any other JPEG.
fn decode_ycca_jpeg(bytes: &[u8]) -> Result<Option<Sprite>, String> {
    use zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
    use zune_jpeg::JpegDecoder;
    if !bytes.starts_with(&[0xff, 0xd8]) {
        return Ok(None);
    }
    let mut probe = JpegDecoder::new(ZCursor::new(bytes));
    probe.decode_headers().map_err(|e| e.to_string())?;
    let info = probe.info().ok_or("no JPEG header")?;
    if info.components != 4 {
        return Ok(None);
    }
    // Asking for "CMYK" output hands back the four channels as stored, unconverted.
    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::CMYK);
    let raw = JpegDecoder::new_with_options(ZCursor::new(bytes), options).decode().map_err(|e| e.to_string())?;
    let pixels = raw.chunks_exact(4).map(|c| ycca_to_argb(c[0], c[1], c[2], c[3])).collect();
    Ok(Some(Sprite { min_x: 0, min_y: 0, width: info.width as usize, height: info.height as usize, pixels }))
}

/// JFIF YCbCr to RGB, with the fourth channel as alpha.
fn ycca_to_argb(y: u8, cb: u8, cr: u8, a: u8) -> u32 {
    let (y, cb, cr) = (y as f32, cb as f32 - 128.0, cr as f32 - 128.0);
    let channel = |v: f32| v.round().clamp(0.0, 255.0) as u32;
    let r = channel(y + 1.402 * cr);
    let g = channel(y - 0.344_136 * cb - 0.714_136 * cr);
    let b = channel(y + 1.772 * cb);
    (a as u32) << 24 | r << 16 | g << 8 | b
}

/// `Class52.method1078` state: smooths the bar between the reported percentages.
#[derive(Default)]
pub struct BarClock {
    last_percent: i32,
    changed_at: Option<Instant>,
}

impl BarClock {
    /// The fill in hundredths of a percent.
    fn fill(&mut self, status: &Status) -> i32 {
        let now = Instant::now();
        let p = status.percent;
        let mut fill = p * 100;
        match self.changed_at {
            Some(changed) if p == self.last_percent && p != 0 => {
                let target = status.target();
                if target > p {
                    let lead = changed.saturating_duration_since(status.started).as_millis() as i64;
                    if lead > 0 {
                        let expected = 10_000 * lead / p as i64 * (target - p) as i64;
                        let elapsed = now.duration_since(changed).as_millis() as i64 * 10_000;
                        fill = if elapsed < expected {
                            (100 * elapsed * (target - p) as i64 / expected) as i32 + p * 100
                        } else {
                            target * 100
                        };
                    }
                }
            }
            _ => {
                self.last_percent = p;
                self.changed_at = Some(now);
            }
        }
        fill
    }

    fn stalled(&self) -> bool {
        self.changed_at.is_some_and(|t| t.elapsed().as_millis() > STALL_MS)
    }
}

/// `Class299.method187`: every element in order.
pub fn draw_screen(r: &mut Raster, screen: &Screen, assets: &Assets, status: &Status, frames: u32, mut clock: Option<&mut BarClock>) {
    let (sw, sh) = (WIDTH as i32, HEIGHT as i32);
    for element in &screen.elements {
        match element {
            Element::Image { image, h_anchor, v_anchor, x, y } => {
                if let Some(s) = assets.images.get(image) {
                    let (w, h) = (s.width as i32, s.height as i32);
                    r.draw_sprite(s, h_anchor.offset(w, sw) + x, v_anchor.offset(h, sh) + y);
                }
            }
            Element::Text { text, h_anchor, v_anchor, x, y, h_align, v_align, line_height, width, height, font, colour, shadow } => {
                if let Some(f) = assets.fonts.get(font) {
                    let bx = h_anchor.offset(*width, sw) + x;
                    let by = v_anchor.offset(*height, sh) + y;
                    f.draw_paragraph(
                        r,
                        text,
                        (bx, by, *width, *height),
                        *colour,
                        *shadow,
                        HAlign::from_code(*h_align),
                        VAlign::from_code(*v_align),
                        *line_height,
                    );
                }
            }
            Element::ImageBar { bar, images, scroll_speed } => {
                let fill = match clock.as_deref_mut() {
                    Some(c) => c.fill(status),
                    None => status.percent * 100,
                };
                let stalled = clock.as_deref().is_some_and(BarClock::stalled);
                draw_image_bar(r, bar, images, *scroll_speed, assets, status, fill, stalled, frames);
            }
            Element::Unported { .. } => {}
        }
    }
}

/// `Class52.method20` with `Class52_Sub2.method1081` (frame) and `method1077` (fill).
#[allow(clippy::too_many_arguments)] // one call site; the arguments are the bar's whole state
fn draw_image_bar(
    r: &mut Raster,
    bar: &BarBox,
    img: &BarImages,
    scroll_speed: i32,
    assets: &Assets,
    status: &Status,
    fill: i32,
    stalled: bool,
    frames: u32,
) {
    let get = |id: i32| assets.images.get(&id);
    let (Some(fill_img), Some(empty), Some(left), Some(right), Some(top), Some(bottom)) =
        (get(img.fill), get(img.empty), get(img.left), get(img.right), get(img.top), get(img.bottom))
    else {
        return;
    };
    let (w, h) = (bar.width, bar.height);
    let x = bar.h_anchor.offset(w, WIDTH as i32) + bar.x;
    let y = bar.v_anchor.offset(h, HEIGHT as i32) + bar.y;
    let size = |s: &Sprite| (s.width as i32, s.height as i32);
    let ((lw, lh), (rw, rh), (_, th), (_, bh)) = (size(left), size(right), size(top), size(bottom));

    // Frame: end caps centred vertically, top and bottom edges tiled between them.
    let saved = r.clip();
    r.set_clip(x, y, x + w, y + h);
    r.draw_sprite(left, x, y + (h - lh) / 2);
    r.draw_sprite(right, x + w - rw, y + (h - rh) / 2);
    r.set_clip(x, y, x + w, y + th);
    r.draw_sprite_tiled(top, x + lw, y, w - lw - rw, h);
    r.set_clip(x, y + h - bh, x + w, y + h);
    r.draw_sprite_tiled(bottom, x + lw, y + h - bh, w - lw - rw, h);
    r.restore_clip(saved);

    // Fill: the filled part tiled with the (scrolling) fill image, the rest with the empty image.
    let (inner_left, inner_right) = (x + lw, x + w - rw);
    let (inner_top, inner_bottom) = (y + th, y + h - bh);
    let (iw, ih) = (inner_right - inner_left, inner_bottom - inner_top);
    let filled = fill * iw / 10_000;
    r.set_clip(inner_left, inner_top, inner_left + filled, inner_bottom);
    let fw = fill_img.width as i32;
    let scroll = if fw > 0 { (scroll_speed as i64 * frames as i64 / 10).rem_euclid(fw as i64) as i32 } else { 0 };
    r.draw_sprite_tiled(fill_img, scroll + (inner_left - fw), inner_top, iw + fw - scroll, ih);
    r.set_clip(inner_left + filled, inner_top, inner_right, inner_bottom);
    r.draw_sprite_tiled(empty, inner_left, inner_top, iw, ih);
    r.restore_clip(saved);

    // Status text, centred, with the stage number once the bar has stalled.
    if let Some(font) = assets.fonts.get(&bar.font) {
        let text = if stalled { format!("{} ({})", status.text, status.stage) } else { status.text.clone() };
        font.draw_centred(r, &text, w / 2 + x, h / 2 + bar.text_offset + y + BAR_TEXT_DROP, bar.text_colour, -1);
    }
}
