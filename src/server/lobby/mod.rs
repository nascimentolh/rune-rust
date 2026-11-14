//! The lobby: where a player waits between logging in and entering a world.
//!
//! A lobby player shares no state with anyone, so there is no tick here: each one is served on
//! their own connection thread, and a packet is answered as soon as it is read. There is no
//! reference server, so what is sent is this server's own, chosen from what the lobby window's
//! scripts wait for.
//!
//! - `world_list` — the world list the lobby is given
//! - here         — what follows the lobby reply, then reading and answering

mod world_list;

use super::net::Ctx;
use super::session;
use super::worlds::Worlds;
use crate::protocol::client_packets;
use crate::protocol::client_prot::ClientProt;
use crate::protocol::isaac::Isaac;
use crate::protocol::server_packets;
use crate::protocol::server_prot::{self, Packet};
use std::io::{Read, Write};
use std::sync::Arc;

/// How long the membership this server gives lasts from each login: a year.
pub const MEMBERSHIP_MS: i64 = 365 * 24 * 60 * 60 * 1000;

/// Player variable 2567: the lobby window's every-tick script (6347) only builds the lobby
/// (script 1299) once it isn't 0. Its value isn't read otherwise; 1 means "lobby ready".
const LOBBY_READY: (u16, i32) = (2567, 1);

/// Player variable 2411, bits 0..=2 (varbit 10242): which lobby panel shows. Script 2354 (the
/// lobby root's hook on variable 2411) shows the main panel with the tabs and Play for 1, a
/// different one (906:40) for 2, and nothing for 0.
const LOBBY_PANEL: (u16, i32) = (2411, 1);

/// The packets that follow the lobby reply, in order.
fn first_packets() -> Vec<Packet> {
    [LOBBY_READY, LOBBY_PANEL].into_iter().map(|(id, value)| server_packets::varp(id, value)).collect()
}

pub struct Lobby {
    /// The address the world list gives for this server's worlds.
    host: String,
    /// The worlds to list, with their players.
    worlds: Arc<Worlds>,
}

impl Lobby {
    pub fn new(host: String, worlds: Arc<Worlds>) -> Self {
        Lobby { host, worlds }
    }

    /// A lobby player from the login reply until they leave: sends what the lobby window
    /// waits for, then answers each packet as it comes.
    pub fn session<S: Read + Write>(
        &self,
        stream: &mut S,
        ctx: &Ctx,
        (mut from_client, mut to_client): (Isaac, Isaac),
    ) -> std::io::Result<()> {
        let mut send = |stream: &mut S, packets: Vec<Packet>| {
            let bytes: Vec<u8> =
                packets.iter().flat_map(|(packet, payload)| server_prot::frame(*packet, payload, &mut to_client)).collect();
            stream.write_all(&bytes)
        };
        send(stream, first_packets())?;
        while let Some((packet, payload)) = session::read_packet(stream, &mut from_client)? {
            ctx.log(session::describe(packet, &payload));
            if let Some(reply) = self.answer(packet, &payload) {
                send(stream, vec![reply])?;
            }
        }
        Ok(())
    }

    /// The answer to a lobby player's packet, if it has one.
    fn answer(&self, packet: ClientProt, payload: &[u8]) -> Option<Packet> {
        match packet {
            ClientProt::RequestWorldList => {
                client_packets::world_list_held(payload).map(|held| world_list::reply(held, &self.host, &self.worlds.all()))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::server_prot::ServerProt;

    #[test]
    fn the_lobby_answers_a_world_list_request_and_nothing_else() {
        let lobby = Lobby::new("127.0.0.1".into(), Worlds::new(Default::default(), Default::default(), None));
        let (packet, payload) = client_packets::request_world_list(0);
        assert_eq!(lobby.answer(packet, &payload).map(|(reply, _)| reply), Some(ServerProt::WorldList));
        assert_eq!(lobby.answer(packet, &[1]), None);
        assert_eq!(lobby.answer(ClientProt::ScreenSize, &client_packets::screen_size(2, 1, 1, 0).1), None);
        assert_eq!(first_packets().len(), 2);
    }
}
