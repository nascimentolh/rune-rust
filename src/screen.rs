//! The status screen: start-up progress and indexes on the left,
//! the running server (connections, requests, log) on the right.

use crate::loader::Loader;
use crate::net::Server;
use macroquad::prelude::*;

pub const SCREEN_W: f32 = 1024.0;
pub const SCREEN_H: f32 = 640.0;
const BAR: Rect = Rect { x: 20.0, y: 70.0, w: 984.0, h: 30.0 };
const LEFT: Rect = Rect { x: 20.0, y: 180.0, w: 500.0, h: 450.0 };
const RIGHT: Rect = Rect { x: 540.0, y: 180.0, w: 464.0, h: 450.0 };
const BAR_COLOUR: Color = Color::new(0.55, 0.0, 0.0, 1.0);
const PANEL: Color = Color::new(0.08, 0.08, 0.09, 1.0);
const TEXT: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const DIM: Color = Color::new(0.6, 0.6, 0.6, 1.0);
const GOOD: Color = Color::new(0.5, 0.9, 0.5, 1.0);
const ERROR: Color = Color::new(1.0, 0.45, 0.4, 1.0);
const LINE: f32 = 16.0;
const FONT: f32 = 16.0;

/// What was drawn (for the self-test).
#[derive(Default, Debug)]
pub struct Drawn {
    pub progress_bar: bool,
    pub status: bool,
    pub index_lines: usize,
    pub keys_line: bool,
    pub server_panel: bool,
}

pub fn draw(loader: &Loader, server: Option<(&Server, &str)>) -> Drawn {
    clear_background(BLACK);
    let mut drawn = Drawn::default();

    draw_text("Cache server", BAR.x, 32.0, 28.0, WHITE);
    draw_text(format!("Cache: {}", loader.dir.display()), BAR.x, 56.0, FONT, DIM);

    draw_rectangle_lines(BAR.x - 2.0, BAR.y - 2.0, BAR.w + 4.0, BAR.h + 4.0, 2.0, BAR_COLOUR);
    draw_rectangle(BAR.x, BAR.y, BAR.w * loader.progress(), BAR.h, BAR_COLOUR);
    centred(&format!("Loading - {}%", (loader.progress() * 100.0) as u32), BAR.y + 21.0, 20.0, WHITE);
    drawn.progress_bar = true;

    let failed = loader.status().starts_with("Failed");
    draw_text(loader.status(), BAR.x, 128.0, 20.0, if failed { ERROR } else { TEXT });
    drawn.status = true;
    if failed {
        draw_text("Set 'cache' in server.cfg to the folder that holds main_file_cache.dat2", BAR.x, 150.0, FONT, DIM);
    }
    drawn.keys_line = draw_keys_line(loader);
    drawn.index_lines = draw_indexes(loader);
    if let Some((server, address)) = server {
        draw_server(server, address);
        drawn.server_panel = true;
    }
    drawn
}

fn draw_keys_line(loader: &Loader) -> bool {
    let r = &loader.key_report;
    let (text, colour) = match &r.error {
        Some(e) => (format!("Map keys: {e}"), ERROR),
        None if r.total == 0 => return false,
        None => (
            format!("Map keys: {} loaded, {} decrypt correctly, {} wrong, {} not in this cache", r.total, r.ok, r.wrong, r.missing),
            if r.wrong == 0 { GOOD } else { ERROR },
        ),
    };
    draw_text(&text, BAR.x, 152.0, FONT, colour);
    true
}

fn panel(r: Rect, title: &str) {
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_text(title, r.x + 8.0, r.y + 18.0, 18.0, WHITE);
}

fn draw_indexes(loader: &Loader) -> usize {
    panel(LEFT, "Indexes");
    let rows = ((LEFT.h - 34.0) / LINE) as usize;
    let column_w = LEFT.w / 2.0;
    for (n, info) in loader.indexes.iter().enumerate() {
        let x = LEFT.x + 8.0 + (n / rows) as f32 * column_w;
        let y = LEFT.y + 40.0 + (n % rows) as f32 * LINE;
        let (text, colour) = match &info.result {
            Ok((archives, _files, revision)) => (format!("idx{:<3}{archives:>7} archives  r{revision}", info.index), TEXT),
            Err(e) => (format!("idx{:<3}{e}", info.index), ERROR),
        };
        draw_text(&text, x, y, FONT, colour);
    }
    loader.indexes.len()
}

fn draw_server(server: &Server, address: &str) {
    panel(RIGHT, "Server");
    let s = server.stats.lock().unwrap();
    let x = RIGHT.x + 8.0;
    let lines = [
        format!("Listening on {address}"),
        format!("Clients: {} connected now, {} total", s.active, s.connections),
        format!("Archives sent: {}  ({:.1} MB)", s.requests, s.bytes_sent as f64 / 1_048_576.0),
        format!("Logins: {}", s.logins),
    ];
    for (n, line) in lines.iter().enumerate() {
        draw_text(line, x, RIGHT.y + 42.0 + n as f32 * 20.0, 18.0, if n == 0 { GOOD } else { TEXT });
    }
    let log_top = RIGHT.y + 132.0;
    let visible = ((RIGHT.bottom() - log_top) / LINE) as usize;
    for (n, line) in s.log.iter().rev().take(visible).rev().enumerate() {
        draw_text(line, x, log_top + n as f32 * LINE, 15.0, DIM);
    }
}

fn centred(text: &str, y: f32, size: f32, colour: Color) {
    let w = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, (SCREEN_W - w) / 2.0, y, size, colour);
}
