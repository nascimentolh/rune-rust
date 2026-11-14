//! The administrator's part of the status window: the worlds that are running, and a form to
//! open another one. Every control here stands for something the client is told about a world
//! (`super::worlds::Options`), so nothing can be set that no one ever sees.
//!
//! The Flag field is the one that isn't a plain value: a country is a number to the client, so
//! clicking the field opens the flags of the cache (`super::flags`) to pick one by its picture.

use super::flags::Flag;
use super::net::Server;
use super::screen::{DIM, ERROR, GOOD, PANEL, TEXT};
use super::worlds::{Options, NAME_LEN};
use macroquad::prelude::*;
use std::sync::Arc;

const AREA: Rect = Rect { x: 20.0, y: 576.0, w: 984.0, h: 224.0 };
const TABLE_X: f32 = AREA.x + 12.0;
const DIVIDER_X: f32 = AREA.x + 536.0;
const FORM_X: f32 = AREA.x + 552.0;
/// Where a form field starts; its label sits 72 to the left.
const FIELD_X: f32 = FORM_X + 78.0;
const LABEL_BACK: f32 = 72.0;
const FIELD_H: f32 = 22.0;
const BOX: f32 = 14.0;
const OPEN: Rect = Rect { x: FIELD_X, y: AREA.y + 176.0, w: 150.0, h: 28.0 };

/// The flags to pick from, over the panels above while it is open. It is only as big as the
/// flags it holds, up to `PICKER_MAX`.
const PICKER_X: f32 = 20.0;
const PICKER_Y: f32 = 180.0;
const PICKER_MAX: Vec2 = Vec2::new(984.0, 380.0);
const CELL_W: f32 = 54.0;
const CELL_H: f32 = 38.0;
/// Where the grid starts inside the picker, and what is left under it.
const GRID_X: f32 = PICKER_X + 10.0;
const GRID_Y: f32 = PICKER_Y + 34.0;
const GRID_BOTTOM: f32 = 12.0;

const FIELD: Color = Color::new(0.14, 0.14, 0.16, 1.0);
const EDGE: Color = Color::new(0.28, 0.28, 0.31, 1.0);
const ACCENT: Color = Color::new(0.72, 0.12, 0.12, 1.0);
const ACCENT_LIT: Color = Color::new(0.85, 0.20, 0.20, 1.0);
const HEADING: Color = Color::new(0.78, 0.74, 0.62, 1.0);
const STRIPE: Color = Color::new(0.11, 0.11, 0.13, 1.0);
const OVER: Color = Color::new(0.06, 0.06, 0.07, 1.0);
const LINE: f32 = 17.0;
const FONT: f32 = 15.0;
/// Worlds listed before the rest are only counted.
const ROWS: usize = 7;

/// The text fields: (label, x from `FIELD_X`, y from `AREA.y`, width, longest, digits only).
const FIELDS: [(&str, f32, f32, f32, usize, bool); 5] = [
    ("Name", 0.0, 36.0, 246.0, NAME_LEN, false),
    ("Location", 0.0, 64.0, 150.0, NAME_LEN, false),
    ("Flag", 234.0, 64.0, 60.0, 5, true),
    ("Port", 0.0, 92.0, 90.0, 5, true),
    ("Players", 234.0, 92.0, 60.0, 4, true),
];
const NAME: usize = 0;
const LOCATION: usize = 1;
const FLAG: usize = 2;
const PORT: usize = 3;
const PLAYERS: usize = 4;

/// The tick boxes: (label, x from `FIELD_X`, y from `AREA.y`).
const BOXES: [(&str, f32, f32); 6] = [
    ("Members", 0.0, 120.0),
    ("Loot share", 110.0, 120.0),
    ("Highlighted", 212.0, 120.0),
    ("Quick chat", 0.0, 146.0),
    ("Bots", 110.0, 146.0),
    ("Hidden", 212.0, 146.0),
];
const MEMBERS: usize = 0;
const LOOTSHARE: usize = 1;
const HIGHLIGHTED: usize = 2;
const QUICKCHAT: usize = 3;
const BOTS: usize = 4;
const HIDDEN: usize = 5;

fn field_rect(n: usize) -> Rect {
    let (_, x, y, w, _, _) = FIELDS[n];
    Rect { x: FIELD_X + x, y: AREA.y + y, w, h: FIELD_H }
}

fn box_rect(n: usize) -> Rect {
    let (_, x, y) = BOXES[n];
    Rect { x: FIELD_X + x, y: AREA.y + y, w: BOX, h: BOX }
}

/// How many flags the picker puts across and down for `flags` of them: as many as fit, and
/// never more rows than the flags need.
fn grid(flags: usize) -> (usize, usize) {
    let columns = ((PICKER_MAX.x - 20.0) / CELL_W) as usize;
    let most = ((PICKER_MAX.y - (GRID_Y - PICKER_Y) - GRID_BOTTOM) / CELL_H) as usize;
    (columns, flags.div_ceil(columns.max(1)).clamp(1, most))
}

/// The picker around a grid of `flags` flags.
fn picker(flags: usize) -> Rect {
    let (columns, rows) = grid(flags);
    let across = flags.clamp(1, columns) as f32 * CELL_W + 20.0;
    Rect { x: PICKER_X, y: PICKER_Y, w: across, h: GRID_Y - PICKER_Y + rows as f32 * CELL_H + GRID_BOTTOM }
}

/// Where flag number `n` sits while the grid is scrolled down by `scroll` rows.
fn cell(n: usize, scroll: usize, flags: usize) -> Rect {
    let (columns, _) = grid(flags);
    let (column, row) = (n % columns, n / columns);
    let y = GRID_Y + (row as f32 - scroll as f32) * CELL_H;
    Rect { x: GRID_X + column as f32 * CELL_W, y, w: CELL_W, h: CELL_H }
}

/// The flag under `at`, if the grid has one there.
fn cell_at(at: Vec2, flags: usize, scroll: usize) -> Option<usize> {
    let (columns, rows) = grid(flags);
    let (column, row) = ((at.x - GRID_X) / CELL_W, (at.y - GRID_Y) / CELL_H);
    if at.x < GRID_X || at.y < GRID_Y || column >= columns as f32 || row >= rows as f32 {
        return None;
    }
    let n = (row as usize + scroll) * columns + column as usize;
    (n < flags).then_some(n)
}

/// What the administrator has typed and ticked, and what the last attempt said.
pub struct Panel {
    text: [String; FIELDS.len()],
    flags: [bool; BOXES.len()],
    focus: usize,
    /// The flags are open to pick from, scrolled down by `scroll` rows.
    picking: bool,
    scroll: usize,
    /// One picture per country flag, made as it is first drawn.
    pictures: Vec<Option<Texture2D>>,
    /// The last result, and whether it was a success.
    message: Option<(String, bool)>,
}

impl Default for Panel {
    fn default() -> Self {
        let start = Options::default();
        let mut text: [String; FIELDS.len()] = Default::default();
        text[LOCATION] = start.location.clone();
        text[FLAG] = start.country.to_string();
        text[PLAYERS] = start.capacity.to_string();
        let mut flags = [false; BOXES.len()];
        flags[MEMBERS] = start.members;
        Panel { text, flags, focus: NAME, picking: false, scroll: 0, pictures: Vec::new(), message: None }
    }
}

impl Panel {
    /// The options the form holds; `free_port` is asked when the port was left empty.
    fn options(&self, free_port: impl FnOnce() -> Option<u16>) -> Result<Options, String> {
        let typed = |n: usize| self.text[n].trim();
        let port = match typed(PORT) {
            "" => free_port().ok_or_else(|| "no free port was found".to_string())?,
            port => port.parse().map_err(|_| "the port must be a number up to 65535".to_string())?,
        };
        let options = Options {
            name: typed(NAME).to_string(),
            port,
            capacity: typed(PLAYERS).parse().map_err(|_| "say how many players the world holds".to_string())?,
            members: self.flags[MEMBERS],
            lootshare: self.flags[LOOTSHARE],
            highlighted: self.flags[HIGHLIGHTED],
            hidden: self.flags[HIDDEN],
            quickchat: self.flags[QUICKCHAT],
            bots_allowed: self.flags[BOTS],
            country: match typed(FLAG) {
                "" => 0,
                flag => flag.parse().map_err(|_| "the flag must be a number".to_string())?,
            },
            location: typed(LOCATION).to_string(),
        };
        options.check().map(|()| options)
    }

    /// A typed character goes to the field that has the focus, if it fits there.
    fn type_char(&mut self, c: char) {
        let (_, _, _, _, longest, digits) = FIELDS[self.focus];
        let fits = if digits { c.is_ascii_digit() } else { c.is_ascii_graphic() || c == ' ' };
        if fits && self.text[self.focus].chars().count() < longest {
            self.text[self.focus].push(c);
        }
    }

    /// Opens the world the form describes and says how it went.
    fn open(&mut self, server: &Arc<Server>) {
        let opened = self.options(|| server.free_port()).and_then(|options| server.open_world(options));
        self.message = Some(match opened {
            Ok(world) => {
                // The next world starts from this one's settings, with a name and port of its own.
                self.text[NAME].clear();
                self.text[PORT].clear();
                let open = format!("World {} is open on port {}", world.number, world.options.port);
                match server.worlds.save() {
                    Ok(()) => (open, true),
                    Err(e) => (format!("{open}, but {e}"), false),
                }
            }
            Err(e) => (e, false),
        });
    }

    /// This frame's mouse and keyboard.
    pub fn update(&mut self, server: &Arc<Server>) {
        while let Some(c) = get_char_pressed() {
            self.type_char(c);
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.text[self.focus].pop();
        }
        if is_key_pressed(KeyCode::Tab) {
            self.focus = (self.focus + 1) % FIELDS.len();
            self.picking = false;
        }
        if self.picking {
            return self.pick(server);
        }
        let mut open = is_key_pressed(KeyCode::Enter);
        if is_mouse_button_pressed(MouseButton::Left) {
            let at = Vec2::from(mouse_position());
            for n in 0..FIELDS.len() {
                if field_rect(n).contains(at) {
                    self.focus = n;
                    // A country is a number to the client, so its flags are worth seeing.
                    self.picking = n == FLAG;
                }
            }
            for n in 0..BOXES.len() {
                self.flags[n] ^= box_rect(n).contains(at);
            }
            open |= OPEN.contains(at);
        }
        if open {
            self.open(server);
        }
    }

    /// The mouse and keyboard while the flags are open.
    fn pick(&mut self, server: &Server) {
        let (columns, rows) = grid(server.flags.len());
        let last_row = server.flags.len().div_ceil(columns.max(1));
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            let by = if wheel > 0.0 { -1 } else { 1 };
            self.scroll = (self.scroll as i64 + by).clamp(0, last_row.saturating_sub(rows) as i64) as usize;
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            self.picking = false;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let at = Vec2::from(mouse_position());
            if let Some(n) = cell_at(at, server.flags.len(), self.scroll) {
                self.text[FLAG] = server.flags[n].country.to_string();
                self.picking = false;
            } else if !picker(server.flags.len()).contains(at) {
                // Clicking away from the flags puts them away.
                self.picking = false;
            }
        }
    }

    pub fn draw(&mut self, server: &Server) {
        let at = Vec2::from(mouse_position());
        draw_rectangle(AREA.x, AREA.y, AREA.w, AREA.h, PANEL);
        draw_line(DIVIDER_X, AREA.y + 10.0, DIVIDER_X, AREA.bottom() - 10.0, 1.0, EDGE);
        self.draw_worlds(server);
        self.draw_form(server, at);
        if self.picking {
            self.draw_picker(server, at);
        }
    }

    /// The worlds that are running, newest last.
    fn draw_worlds(&self, server: &Server) {
        let worlds = server.worlds.all();
        draw_text("Worlds", TABLE_X, AREA.y + 20.0, 18.0, WHITE);
        draw_text(row("#", "World", "Port", "Players", "Type", "Flag", "Tick"), TABLE_X, AREA.y + 42.0, FONT, HEADING);
        let stats = server.stats.lock().unwrap();
        for (n, hosted) in worlds.iter().take(ROWS).enumerate() {
            let options = &hosted.options;
            let y = AREA.y + 62.0 + n as f32 * LINE;
            if n % 2 == 1 {
                draw_rectangle(TABLE_X - 6.0, y - 12.0, DIVIDER_X - TABLE_X - 4.0, LINE, STRIPE);
            }
            // The client shows a world's name in place of its flag and location, so the table
            // shows the location in brackets when the world has no name.
            let name = if options.name.is_empty() { format!("[{}]", options.location) } else { options.name.clone() };
            let flags: String = [
                (options.lootshare, 'L'),
                (options.highlighted, 'H'),
                (options.quickchat, 'Q'),
                (options.bots_allowed, 'B'),
                (options.hidden, '-'),
            ]
            .iter()
            .filter(|(on, _)| *on)
            .map(|(_, letter)| *letter)
            .collect();
            // The slowest tick this world has had, in seconds once milliseconds run out of room.
            let ms = stats.worlds.get(&hosted.number).map_or(0.0, |w| w.slowest_tick.as_secs_f64() * 1000.0);
            let tick = if ms >= 1000.0 { format!("{:.1} s", ms / 1000.0) } else { format!("{ms:.1} ms") };
            let line = row(
                &hosted.number.to_string(),
                &name,
                &options.port.to_string(),
                &format!("{}/{}", hosted.world.seats().online(), hosted.world.seats().capacity()),
                if options.members { "Members" } else { "Free" },
                if flags.is_empty() { "-" } else { &flags },
                &tick,
            );
            draw_text(&line, TABLE_X, y, FONT, if options.hidden { DIM } else { TEXT });
        }
        if worlds.len() > ROWS {
            let y = AREA.y + 62.0 + ROWS as f32 * LINE;
            draw_text(format!("and {} more", worlds.len() - ROWS), TABLE_X, y, FONT, DIM);
        }
        draw_text("A world's name shows in place of its flag and location.", TABLE_X, AREA.bottom() - 12.0, 14.0, DIM);
    }

    /// The form that opens a world.
    fn draw_form(&mut self, server: &Server, at: Vec2) {
        draw_text("Open a world", FORM_X, AREA.y + 20.0, 18.0, WHITE);
        for (n, (label, _, _, _, _, _)) in FIELDS.iter().enumerate() {
            let rect = field_rect(n);
            let focused = self.focus == n;
            draw_text(label, rect.x - LABEL_BACK, rect.y + 16.0, FONT, DIM);
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, if focused { ACCENT_LIT } else { EDGE });
            draw_text(&self.text[n], rect.x + 6.0, rect.y + 16.0, FONT, TEXT);
            // The caret blinks at the end of the field that has the focus.
            if focused && (get_time() * 2.0) as i64 % 2 == 0 {
                let width = measure_text(&self.text[n], None, FONT as u16, 1.0).width;
                draw_text("_", rect.x + 7.0 + width, rect.y + 16.0, FONT, ACCENT_LIT);
            }
        }
        if self.text[PORT].trim().is_empty() {
            let rect = field_rect(PORT);
            draw_text("next free", rect.x + 6.0, rect.y + 16.0, FONT, EDGE);
        }
        // The flag the typed country stands for, beside its field.
        let rect = field_rect(FLAG);
        let country: Option<i32> = self.text[FLAG].trim().parse().ok();
        match country.and_then(|country| server.flags.iter().position(|flag| flag.country == country)) {
            Some(n) => self.draw_flag(&server.flags, n, Rect { x: rect.right() + 6.0, y: rect.y, w: 52.0, h: rect.h }),
            None => {
                draw_text("no flag", rect.right() + 8.0, rect.y + 16.0, 14.0, EDGE);
            }
        }
        for (n, (label, _, _)) in BOXES.iter().enumerate() {
            let rect = box_rect(n);
            let on = self.flags[n];
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, if rect.contains(at) { ACCENT_LIT } else { EDGE });
            if on {
                draw_line(rect.x + 3.0, rect.y + 7.5, rect.x + 6.0, rect.y + 10.5, 2.0, GOOD);
                draw_line(rect.x + 6.0, rect.y + 10.5, rect.x + 11.0, rect.y + 3.5, 2.0, GOOD);
            }
            draw_text(label, rect.x + rect.w + 7.0, rect.y + 12.0, FONT, if on { TEXT } else { DIM });
        }
        let hot = OPEN.contains(at);
        draw_rectangle(OPEN.x, OPEN.y, OPEN.w, OPEN.h, if hot { ACCENT_LIT } else { ACCENT });
        draw_rectangle_lines(OPEN.x, OPEN.y, OPEN.w, OPEN.h, 1.0, if hot { WHITE } else { ACCENT_LIT });
        draw_text("Open world", OPEN.x + 30.0, OPEN.y + 19.0, 16.0, WHITE);
        if let Some((text, ok)) = &self.message {
            draw_text(text, FORM_X, AREA.bottom() - 12.0, 14.0, if *ok { GOOD } else { ERROR });
        }
    }

    /// The flags of the cache, to pick one by looking at it.
    fn draw_picker(&mut self, server: &Server, at: Vec2) {
        let (columns, rows) = grid(server.flags.len());
        let picker = picker(server.flags.len());
        draw_rectangle(picker.x, picker.y, picker.w, picker.h, OVER);
        draw_rectangle_lines(picker.x, picker.y, picker.w, picker.h, 2.0, ACCENT);
        let hovered = cell_at(at, server.flags.len(), self.scroll);
        let title = match hovered.map(|n| server.flags[n].country) {
            Some(country) => format!("Pick a flag - country {country}"),
            None => "Pick a flag, type its number, or press Escape".to_string(),
        };
        draw_text(&title, picker.x + 10.0, picker.y + 22.0, 16.0, WHITE);
        let shown = self.scroll * columns..((self.scroll + rows) * columns).min(server.flags.len());
        if server.flags.len() > columns * rows {
            let more = format!("{} to {} of {} - mouse wheel for more", shown.start + 1, shown.end, server.flags.len());
            draw_text(&more, picker.right() - 10.0 - measure_text(&more, None, 14, 1.0).width, picker.y + 22.0, 14.0, DIM);
        }
        for n in shown {
            let cell = cell(n, self.scroll, server.flags.len());
            if hovered == Some(n) {
                draw_rectangle(cell.x, cell.y, cell.w, cell.h, STRIPE);
                draw_rectangle_lines(cell.x, cell.y, cell.w, cell.h, 1.0, ACCENT_LIT);
            }
            self.draw_flag(&server.flags, n, cell);
        }
    }

    /// One flag, as big as it fits inside `space` without stretching.
    fn draw_flag(&mut self, flags: &[Flag], n: usize, space: Rect) {
        if self.pictures.len() != flags.len() {
            self.pictures = (0..flags.len()).map(|_| None).collect();
        }
        let flag = &flags[n];
        let picture = self.pictures[n].get_or_insert_with(|| {
            let picture = Texture2D::from_rgba8(flag.width, flag.height, &flag.rgba);
            picture.set_filter(FilterMode::Nearest);
            picture
        });
        let (w, h) = (f32::from(flag.width), f32::from(flag.height));
        let scale = ((space.w - 8.0) / w).min((space.h - 8.0) / h).clamp(0.1, 2.0);
        let size = vec2(w * scale, h * scale);
        let at = vec2(space.x + (space.w - size.x) / 2.0, space.y + (space.h - size.y) / 2.0);
        draw_texture_ex(picture, at.x, at.y, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
    }
}

/// One line of the worlds table. Each column is cut to its own width, so nothing a world is
/// named or does can push the columns out of line.
fn row(number: &str, name: &str, port: &str, players: &str, kind: &str, flags: &str, tick: &str) -> String {
    let cut = |text: &str, width: usize| -> String { text.chars().take(width).collect() };
    format!(
        "{:>2}  {:<16} {:>5}  {:>9}  {:<7} {:<4} {:>8}",
        cut(number, 2),
        cut(name, 16),
        cut(port, 5),
        cut(players, 9),
        cut(kind, 7),
        cut(flags, 4),
        cut(tick, 8)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(panel: &mut Panel, field: usize, text: &str) {
        panel.focus = field;
        text.chars().for_each(|c| panel.type_char(c));
    }

    #[test]
    fn the_form_becomes_a_worlds_options() {
        let mut panel = Panel::default();
        typed(&mut panel, NAME, " Trade world ");
        // Nothing else touched: the port is the server's next free one and the rest are the
        // defaults the form starts with.
        assert_eq!(panel.options(|| Some(5556)), Ok(Options { name: "Trade world".into(), port: 5556, ..Options::default() }));
        typed(&mut panel, PORT, "6000x");
        typed(&mut panel, FLAG, "2");
        panel.flags[LOOTSHARE] = true;
        panel.flags[QUICKCHAT] = true;
        let options = panel.options(|| None).unwrap();
        // "6000x": the x isn't a digit, so it never reached the field.
        assert_eq!((options.port, options.country), (6000, 2));
        assert_eq!((options.lootshare, options.quickchat, options.members), (true, true, true));
    }

    #[test]
    fn what_cannot_be_a_world_is_explained() {
        let mut panel = Panel::default();
        // No port typed and none free.
        assert!(panel.options(|| None).is_err());
        typed(&mut panel, PORT, "99999");
        assert!(panel.options(|| None).is_err());
        panel.text[PORT] = "6000".into();
        panel.text[PLAYERS].clear();
        assert!(panel.options(|| None).is_err());
        typed(&mut panel, PLAYERS, "9999");
        assert!(panel.options(|| None).is_err());
        // A name stops at its longest, and the number fields take digits only.
        typed(&mut panel, NAME, &"n".repeat(NAME_LEN + 5));
        assert_eq!((panel.text[NAME].len(), panel.text[PLAYERS].as_str()), (NAME_LEN, "9999"));
    }

    /// Every control is inside the panel, and none of them overlaps another.
    #[test]
    fn the_controls_are_laid_out_inside_the_panel() {
        let mut rects: Vec<Rect> = (0..FIELDS.len()).map(field_rect).collect();
        rects.extend((0..BOXES.len()).map(box_rect));
        rects.push(OPEN);
        for (n, rect) in rects.iter().enumerate() {
            assert!(AREA.contains(rect.point()) && AREA.contains(vec2(rect.right(), rect.bottom())), "{rect:?} is outside the panel");
            assert!(rect.x > DIVIDER_X, "{rect:?} is over the worlds table");
            for other in &rects[n + 1..] {
                assert!(rect.intersect(*other).is_none(), "{rect:?} and {other:?} overlap");
            }
        }
    }

    /// The heading and a row line up column for column.
    #[test]
    fn the_table_columns_have_one_width() {
        let heading = row("#", "World", "Port", "Players", "Type", "Flag", "Tick");
        // Every column at its widest, and then wider than that: both line up with the heading.
        let full = row("12", "[United Kingdom]", "65535", "2000/2000", "Members", "LTHQ", "999.9 ms");
        let over = row("123", &"n".repeat(NAME_LEN), "655350", "20000/20000", "Members!", "LTHQ!", "99999.9 s");
        assert_eq!((full.len(), over.len()), (heading.len(), heading.len()));
    }

    /// Every flag the picker draws is where the click that picks it lands, and the ones
    /// scrolled past or after the last flag belong to nobody.
    #[test]
    fn the_picker_finds_the_flag_that_was_clicked() {
        let (columns, most_rows) = grid(usize::MAX);
        assert!(columns > 4 && most_rows > 2, "the picker shows {columns} by {most_rows} flags");
        // More flags than fit, and the handful this cache really has.
        for flags in [columns * (most_rows + 3), 24] {
            let (_, rows) = grid(flags);
            let picker = picker(flags);
            for scroll in [0, 1] {
                for n in scroll * columns..((scroll + rows) * columns).min(flags) {
                    let middle = cell(n, scroll, flags).center();
                    assert!(picker.contains(middle), "flag {n} of {flags} is drawn outside the picker");
                    assert_eq!(cell_at(middle, flags, scroll), Some(n));
                }
                // A row that has been scrolled past, and the space after the last flag.
                if scroll > 0 && flags > columns * rows {
                    assert_eq!(cell_at(cell(0, scroll, flags).center(), flags, scroll), None);
                }
                assert_eq!(cell_at(cell(flags, scroll, flags).center(), flags, scroll), None);
            }
        }
    }
}
