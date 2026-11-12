//! What the server sends a player right after a lobby login. There is no reference server, so
//! the contents are this server's own, chosen from what the lobby window's scripts wait for.

use super::server_prot;

/// Player variable 2567: the lobby window's every-tick script (6347) only builds the lobby
/// (script 1299) once it isn't 0. Its value isn't read otherwise; 1 means "lobby ready".
const LOBBY_READY: (u16, i32) = (2567, 1);

/// The packets (id, payload) that follow the lobby reply, in order.
pub fn first_packets() -> Vec<(u32, Vec<u8>)> {
    let (id, value) = LOBBY_READY;
    vec![server_prot::varp(id, value)]
}
