//! A player as the world keeps them: plain game state, with no socket and no lock, so every
//! handler and tick stage can be tested by making a `Player` and looking at what it queued.

use super::interfaces;
use crate::protocol::server_packets;
use crate::protocol::server_prot::Packet;

pub struct Player {
    /// Their player index (1..=2047), from their seat.
    pub index: u16,
    pub name: String,
    /// The game's window they have open (548 or 746).
    pub window: u16,
    /// Set when they are to be let go at the end of this tick, after what is queued is sent.
    pub leaving: bool,
    /// Packets for them, in order; sent together at the end of the tick.
    queued: Vec<Packet>,
}

impl Player {
    pub fn new(index: u16, name: String, window_mode: Option<u8>) -> Self {
        Player { index, name, window: interfaces::game_window(window_mode), leaving: false, queued: Vec::new() }
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
