//! Drawing helpers and colours shared by the client's screens.

use macroquad::prelude::*;

/// The client's fixed window size.
pub const WIDTH: f32 = 765.0;
pub const HEIGHT: f32 = 503.0;

pub const TEXT: Color = Color::new(0.95, 0.92, 0.80, 1.0);
pub const DIM: Color = Color::new(0.65, 0.62, 0.55, 1.0);
pub const GOLD: Color = Color::new(1.0, 0.85, 0.35, 1.0);
pub const ERROR: Color = Color::new(1.0, 0.45, 0.4, 1.0);
pub const GOOD: Color = Color::new(0.55, 0.95, 0.55, 1.0);
const PANEL: Color = Color::new(0.10, 0.09, 0.08, 0.92);
const PANEL_EDGE: Color = Color::new(0.45, 0.36, 0.22, 1.0);
const FIELD: Color = Color::new(0.03, 0.03, 0.03, 1.0);

pub fn centred(text: &str, cx: f32, y: f32, size: f32, colour: Color) {
    let w = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, cx - w / 2.0, y + 1.0, size, BLACK);
    draw_text(text, cx - w / 2.0, y, size, colour);
}

/// Dark sky with soft drifting mist, behind the login screens.
pub fn background() {
    let bands = 40;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let c = Color::new(0.02 + 0.06 * (1.0 - t), 0.03 + 0.07 * (1.0 - t), 0.08 + 0.12 * (1.0 - t), 1.0);
        draw_rectangle(0.0, t * HEIGHT, WIDTH, HEIGHT / bands as f32 + 1.0, c);
    }
    let time = get_time() as f32;
    for k in 0..6 {
        let x = (k as f32 * 157.0 + time * (6.0 + k as f32)) % (WIDTH + 300.0) - 150.0;
        let y = HEIGHT * (0.55 + 0.07 * k as f32);
        draw_ellipse(x, y, 180.0, 26.0, 0.0, Color::new(0.5, 0.55, 0.65, 0.025));
    }
}

pub fn panel(r: Rect) {
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, PANEL_EDGE);
}

/// A text box; `focused` draws the gold border and a blinking cursor.
pub fn field(r: Rect, text: &str, focused: bool) {
    draw_rectangle(r.x, r.y, r.w, r.h, FIELD);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if focused { GOLD } else { PANEL_EDGE });
    let cursor = if focused && (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
    draw_text(format!("{text}{cursor}"), r.x + 8.0, r.y + r.h * 0.7, 20.0, TEXT);
}

pub fn button(r: Rect, label: &str, enabled: bool) {
    let hover = enabled && r.contains(mouse_position().into());
    let fill = match (enabled, hover) {
        (false, _) => Color::new(0.2, 0.2, 0.2, 1.0),
        (true, true) => Color::new(0.55, 0.36, 0.16, 1.0),
        (true, false) => Color::new(0.42, 0.27, 0.12, 1.0),
    };
    draw_rectangle(r.x, r.y, r.w, r.h, fill);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, PANEL_EDGE);
    centred(label, r.x + r.w / 2.0, r.y + r.h * 0.68, 22.0, if enabled { GOLD } else { DIM });
}
