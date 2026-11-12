//! What the server sends a player right after a lobby login. There is no reference server, so
//! the contents are this server's own, chosen from what the lobby window's scripts wait for.

use super::server_prot;

/// How long the membership this server gives lasts from each login: a year.
pub const MEMBERSHIP_MS: i64 = 365 * 24 * 60 * 60 * 1000;

/// Player variable 2567: the lobby window's every-tick script (6347) only builds the lobby
/// (script 1299) once it isn't 0. Its value isn't read otherwise; 1 means "lobby ready".
const LOBBY_READY: (u16, i32) = (2567, 1);

/// Player variable 2411, bits 0..=2 (varbit 10242): which lobby panel shows. Script 2354 (the
/// lobby root's hook on variable 2411) shows the main panel with the tabs and Play for 1, a
/// different one (906:40) for 2, and nothing for 0.
const LOBBY_PANEL: (u16, i32) = (2411, 1);

/// The packets (id, payload) that follow the lobby reply, in order.
pub fn first_packets() -> Vec<(u32, Vec<u8>)> {
    [LOBBY_READY, LOBBY_PANEL].into_iter().map(|(id, value)| server_prot::varp(id, value)).collect()
}
