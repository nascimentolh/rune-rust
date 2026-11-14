//! A player as the world keeps them: plain game state, with no socket and no lock, so every
//! handler and tick stage can be tested by making a `Player` and looking at what it queued.

use super::updates::View;
use super::{appearance, enter, interfaces};
use crate::protocol::player_update::Appearance;
use crate::protocol::server_packets;
use crate::protocol::server_prot::Packet;

/// A line of public chat waiting for the tick's chat stage: the effects as `MESSAGE_PUBLIC`
/// carries them (colour << 8 | effect) and the text.
#[derive(Clone, Debug, PartialEq)]
pub struct PublicChat {
    pub effects: u16,
    pub text: String,
}

pub struct Player {
    /// Their player index (1..=2047), from their seat.
    pub index: u16,
    pub name: String,
    /// The game's window they have open (548 or 746).
    pub window: u16,
    /// Set when they are to be let go at the end of this tick, after what is queued is sent.
    pub leaving: bool,
    /// How they look (the appearance mask), and whether it changed since the update stage last
    /// sent it (true on entering: the client has no appearance for them yet).
    pub appearance: Appearance,
    pub appearance_changed: bool,
    /// Where their typed chat goes (`CHAT_TYPE`): 0 public, 1 friends chat, 2 clan, 3 guest
    /// clan. The client sends it around each `CHAT`, which itself doesn't say.
    pub chat_type: u8,
    /// What they said this tick, for the chat stage.
    pub public_chat: Option<PublicChat>,
    /// Their client's player lists, which the update stage keeps in step.
    pub view: View,
    /// Packets for them, in order; sent together at the end of the tick.
    queued: Vec<Packet>,
}

impl Player {
    pub fn new(index: u16, name: String, window_mode: Option<u8>) -> Self {
        let combat_level = appearance::combat_level(|skill| enter::starting_level(skill).1);
        Player {
            index,
            appearance: appearance::new_player(&name, combat_level),
            appearance_changed: true,
            chat_type: 0,
            public_chat: None,
            view: View::new(index),
            name,
            window: interfaces::game_window(window_mode),
            leaving: false,
            queued: Vec::new(),
        }
    }

    /// Queues a packet for the end of the tick.
    pub fn send(&mut self, packet: Packet) {
        self.queued.push(packet);
    }

    pub fn send_all(&mut self, packets: impl IntoIterator<Item = Packet>) {
        self.queued.extend(packets);
    }

    /// Sends them back to the lobby or the login screen and lets them go.
    pub fn log_out(&mut self, to_lobby: bool) {
        self.send(server_packets::logout(to_lobby));
        self.leaving = true;
    }

    /// What is queued, for the flush (or for a test to look at).
    pub fn take_queued(&mut self) -> Vec<Packet> {
        std::mem::take(&mut self.queued)
    }
}
