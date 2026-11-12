//! The login screen: username and password boxes and a Log In button.
//! Keyboard: type, Tab switches box, Enter logs in. Mouse: click a box or the button.

use super::login::{self, Outcome, Pending};
use super::ui::{self, HEIGHT, WIDTH};
use macroquad::prelude::*;

const PANEL: Rect = Rect { x: WIDTH / 2.0 - 190.0, y: 130.0, w: 380.0, h: 270.0 };
const USERNAME: Rect = Rect { x: WIDTH / 2.0 - 140.0, y: 200.0, w: 280.0, h: 32.0 };
const PASSWORD: Rect = Rect { x: WIDTH / 2.0 - 140.0, y: 262.0, w: 280.0, h: 32.0 };
const LOG_IN: Rect = Rect { x: WIDTH / 2.0 - 80.0, y: 314.0, w: 160.0, h: 38.0 };
/// Longest name and password accepted by the boxes. Estimated.
const MAX_USERNAME: usize = 12;
const MAX_PASSWORD: usize = 20;

/// One keyboard action, from the real keyboard or the self-test.
#[derive(Clone, Copy, Debug)]
pub enum Key {
    Char(char),
    Backspace,
    Tab,
    Enter,
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Username,
    Password,
}

pub struct LoginScreen {
    pub username: String,
    password: String,
    focus: Field,
    pub message: (String, Color),
    pending: Option<Pending>,
}

impl LoginScreen {
    pub fn new(message: &str) -> Self {
        LoginScreen {
            username: String::new(),
            password: String::new(),
            focus: Field::Username,
            message: (message.into(), ui::TEXT),
            pending: None,
        }
    }

    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn key(&mut self, key: Key, target: &login::Target) {
        if self.busy() {
            return;
        }
        let (text, max) = match self.focus {
            Field::Username => (&mut self.username, MAX_USERNAME),
            Field::Password => (&mut self.password, MAX_PASSWORD),
        };
        match key {
            Key::Char(c) if !c.is_control() && text.chars().count() < max => text.push(c),
            Key::Char(_) => {}
            Key::Backspace => {
                text.pop();
            }
            Key::Tab => self.focus = if self.focus == Field::Username { Field::Password } else { Field::Username },
            Key::Enter if self.focus == Field::Username => self.focus = Field::Password,
            Key::Enter => self.submit(target),
        }
    }

    pub fn click(&mut self, at: Vec2, target: &login::Target) {
        if USERNAME.contains(at) {
            self.focus = Field::Username;
        } else if PASSWORD.contains(at) {
            self.focus = Field::Password;
        } else if LOG_IN.contains(at) && !self.busy() {
            self.submit(target);
        }
    }

    fn submit(&mut self, target: &login::Target) {
        if self.username.trim().is_empty() || self.password.is_empty() {
            self.message = ("Please enter your username and password.".into(), ui::ERROR);
            return;
        }
        self.message = ("Connecting to server...".into(), ui::TEXT);
        self.pending = Some(login::start(target, self.username.trim(), &self.password));
    }

    /// Checks on a login in progress; returns the outcome once it arrives.
    pub fn poll(&mut self) -> Option<Outcome> {
        let outcome = self.pending.as_ref()?.lock().unwrap().take()?;
        self.pending = None;
        if let Outcome::Refused(why) = &outcome {
            self.message = (why.clone(), ui::ERROR);
            self.password.clear();
        }
        Some(outcome)
    }

    pub fn draw(&self) {
        ui::background();
        ui::panel(PANEL);
        ui::centred("Welcome", WIDTH / 2.0, PANEL.y + 38.0, 30.0, ui::GOLD);
        draw_text("Username:", USERNAME.x, USERNAME.y - 6.0, 18.0, ui::TEXT);
        ui::field(USERNAME, &self.username, self.focus == Field::Username && !self.busy());
        draw_text("Password:", PASSWORD.x, PASSWORD.y - 6.0, 18.0, ui::TEXT);
        ui::field(PASSWORD, &"*".repeat(self.password.chars().count()), self.focus == Field::Password && !self.busy());
        ui::button(LOG_IN, if self.busy() { "Please wait" } else { "Log In" }, !self.busy());
        ui::centred(&self.message.0, WIDTH / 2.0, PANEL.bottom() - 18.0, 16.0, self.message.1);
        ui::centred("Tab: next box    Enter: log in", WIDTH / 2.0, HEIGHT - 14.0, 15.0, ui::DIM);
    }
}
