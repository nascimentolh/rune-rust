//! The administrator's part of the status window: the worlds that are running, and a form to
//! open another one — its name, its port (left empty: the next free one), how many players it
//! holds, and the world list's two flags.

use super::net::Server;
use super::screen::{DIM, ERROR, GOOD, PANEL, TEXT};
use super::worlds::{Options, NAME_LEN};
use macroquad::prelude::*;
use std::sync::Arc;

const AREA: Rect = Rect { x: 20.0, y: 570.0, w: 984.0, h: 180.0 };
const FORM_X: f32 = AREA.x + 560.0;
const NAME: Rect = Rect { x: FORM_X + 70.0, y: AREA.y + 34.0, w: 220.0, h: 22.0 };
const PORT: Rect = Rect { x: FORM_X + 70.0, y: AREA.y + 62.0, w: 70.0, h: 22.0 };
const PLAYERS: Rect = Rect { x: FORM_X + 220.0, y: AREA.y + 62.0, w: 70.0, h: 22.0 };
const MEMBERS: Rect = Rect { x: FORM_X + 70.0, y: AREA.y + 92.0, w: 16.0, h: 16.0 };
const LOOTSHARE: Rect = Rect { x: FORM_X + 190.0, y: AREA.y + 92.0, w: 16.0, h: 16.0 };
const OPEN: Rect = Rect { x: FORM_X + 70.0, y: AREA.y + 118.0, w: 120.0, h: 26.0 };
const FIELD: Color = Color::new(0.16, 0.16, 0.18, 1.0);
const BUTTON: Color = Color::new(0.55, 0.0, 0.0, 1.0);
const LINE: f32 = 16.0;
const FONT: f32 = 16.0;
/// Worlds listed before the rest are only counted.
const ROWS: usize = 8;
/// The players a new world is offered: this server's choice.
const DEFAULT_PLAYERS: &str = "500";

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Name,
    Port,
    Players,
}

/// What the administrator has typed, and what the last attempt said.
pub struct Panel {
    name: String,
    /// Empty = the next free port.
    port: String,
    players: String,
    members: bool,
    lootshare: bool,
    focus: Focus,
    /// The last result, and whether it was a success.
    message: Option<(String, bool)>,
}

impl Default for Panel {
    fn default() -> Self {
        Panel {
            name: String::new(),
            port: String::new(),
            players: DEFAULT_PLAYERS.into(),
            members: true,
            lootshare: false,
            focus: Focus::Name,
            message: None,
        }
    }
}

impl Panel {
    /// The options the form holds; `free_port` is asked when the port was left empty.
    fn options(&self, free_port: impl FnOnce() -> Option<u16>) -> Result<Options, String> {
        let port = match self.port.as_str() {
            "" => free_port().ok_or("no free port was found")?,
            typed => typed.parse().map_err(|_| "the port must be a number up to 65535")?,
        };
        let capacity = self.players.parse().map_err(|_| "say how many players the world holds")?;
        let options = Options { name: self.name.trim().to_string(), port, capacity, members: self.members, lootshare: self.lootshare };
        options.check().map(|()| options)
    }

    /// A typed character goes to the field that has the focus, if it fits there.
    fn type_char(&mut self, c: char) {
        match self.focus {
            Focus::Name if (c.is_ascii_graphic() || c == ' ') && self.name.len() < NAME_LEN => self.name.push(c),
            Focus::Port if c.is_ascii_digit() && self.port.len() < 5 => self.port.push(c),
            Focus::Players if c.is_ascii_digit() && self.players.len() < 4 => self.players.push(c),
            _ => {}
        }
    }

    fn focused(&mut self) -> &mut String {
        match self.focus {
            Focus::Name => &mut self.name,
            Focus::Port => &mut self.port,
            Focus::Players => &mut self.players,
        }
    }

    /// Opens the world the form describes and says how it went.
    fn open(&mut self, server: &Arc<Server>) {
        let opened = self.options(|| server.free_port()).and_then(|options| server.open_world(options));
        self.message = Some(match opened {
            Ok(world) => {
                self.name.clear();
                self.port.clear();
                match server.worlds.save() {
                    Ok(()) => (format!("World {} is open on port {}", world.number, world.options.port), true),
                    Err(e) => (format!("World {} is open on port {}, but {e}", world.number, world.options.port), false),
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
            self.focused().pop();
        }
        if is_key_pressed(KeyCode::Tab) {
            self.focus = match self.focus {
                Focus::Name => Focus::Port,
                Focus::Port => Focus::Players,
                Focus::Players => Focus::Name,
            };
        }
        let mut open = is_key_pressed(KeyCode::Enter);
        if is_mouse_button_pressed(MouseButton::Left) {
            let at = Vec2::from(mouse_position());
            for (rect, focus) in [(NAME, Focus::Name), (PORT, Focus::Port), (PLAYERS, Focus::Players)] {
                if rect.contains(at) {
                    self.focus = focus;
                }
            }
            self.members ^= MEMBERS.contains(at);
            self.lootshare ^= LOOTSHARE.contains(at);
            open |= OPEN.contains(at);
        }
        if open {
            self.open(server);
        }
    }

    pub fn draw(&self, server: &Server) {
        draw_rectangle(AREA.x, AREA.y, AREA.w, AREA.h, PANEL);
        draw_text("Worlds", AREA.x + 8.0, AREA.y + 18.0, 18.0, WHITE);
        let worlds = server.worlds.all();
        let stats = server.stats.lock().unwrap();
        for (n, hosted) in worlds.iter().take(ROWS).enumerate() {
            let o = &hosted.options;
            let name = if o.name.is_empty() { "-" } else { &o.name };
            let flags = format!("{}{}", if o.members { "M" } else { "-" }, if o.lootshare { "L" } else { "-" });
            let tick = stats.worlds.get(&hosted.number).map_or(0.0, |w| w.slowest_tick.as_secs_f64() * 1000.0);
            let line = format!(
                "{:>2}  {:<24}  port {:<5}  {:>4}/{:<4}  {flags}  slowest tick {tick:.1} ms",
                hosted.number,
                name,
                o.port,
                hosted.world.seats().online(),
                hosted.world.seats().capacity()
            );
            draw_text(&line, AREA.x + 8.0, AREA.y + 42.0 + n as f32 * LINE, 15.0, TEXT);
        }
        if worlds.len() > ROWS {
            draw_text(format!("and {} more", worlds.len() - ROWS), AREA.x + 8.0, AREA.y + 42.0 + ROWS as f32 * LINE, 15.0, DIM);
        }

        draw_text("Open a world", FORM_X, AREA.y + 18.0, 18.0, WHITE);
        let field = |rect: Rect, label: &str, text: &str, hint: &str, focus: Focus| {
            draw_text(label, rect.x - 70.0, rect.y + 16.0, FONT, DIM);
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD);
            if self.focus == focus {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, BUTTON);
            }
            let (shown, colour) = if text.is_empty() { (hint, DIM) } else { (text, TEXT) };
            draw_text(shown, rect.x + 5.0, rect.y + 16.0, FONT, colour);
        };
        field(NAME, "Name", &self.name, "", Focus::Name);
        field(PORT, "Port", &self.port, "free", Focus::Port);
        field(PLAYERS, "Players", &self.players, "", Focus::Players);
        let tick_box = |rect: Rect, label: &str, on: bool| {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD);
            if on {
                draw_rectangle(rect.x + 4.0, rect.y + 4.0, rect.w - 8.0, rect.h - 8.0, GOOD);
            }
            draw_text(label, rect.x + rect.w + 6.0, rect.y + 13.0, FONT, TEXT);
        };
        tick_box(MEMBERS, "Members", self.members);
        tick_box(LOOTSHARE, "LootShare", self.lootshare);
        draw_rectangle(OPEN.x, OPEN.y, OPEN.w, OPEN.h, BUTTON);
        draw_text("Open world", OPEN.x + 16.0, OPEN.y + 18.0, FONT, WHITE);
        if let Some((text, ok)) = &self.message {
            draw_text(text, FORM_X, AREA.y + 166.0, 15.0, if *ok { GOOD } else { ERROR });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(panel: &mut Panel, focus: Focus, text: &str) {
        panel.focus = focus;
        text.chars().for_each(|c| panel.type_char(c));
    }

    #[test]
    fn the_form_becomes_a_worlds_options() {
        let mut panel = Panel::default();
        typed(&mut panel, Focus::Name, " Trade world ");
        // No port typed: the server's next free one.
        assert_eq!(
            panel.options(|| Some(5556)),
            Ok(Options { name: "Trade world".into(), port: 5556, capacity: 500, members: true, lootshare: false })
        );
        typed(&mut panel, Focus::Port, "6000x");
        panel.lootshare = true;
        let options = panel.options(|| None).unwrap();
        assert_eq!((options.port, options.lootshare), (6000, true));
    }

    #[test]
    fn what_cannot_be_a_world_is_explained() {
        let mut panel = Panel::default();
        assert!(panel.options(|| None).is_err());
        typed(&mut panel, Focus::Port, "99999");
        assert!(panel.options(|| None).is_err());
        panel.port = "6000".into();
        panel.players.clear();
        assert!(panel.options(|| None).is_err());
        typed(&mut panel, Focus::Players, "9999");
        assert!(panel.options(|| None).is_err());
        // Letters don't go in the number fields, and a name stops at its longest.
        typed(&mut panel, Focus::Name, &"n".repeat(NAME_LEN + 5));
        assert_eq!((panel.name.len(), panel.players.as_str()), (NAME_LEN, "9999"));
    }
}
