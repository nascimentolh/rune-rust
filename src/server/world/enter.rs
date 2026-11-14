//! What a player is sent on entering the world. There is no reference server, so the contents
//! are this server's own, chosen from what the client and the game window's scripts wait for;
//! each choice says so.

use super::interfaces;
use super::player::Player;
use crate::cache::xtea::Key;
use crate::protocol::server_prot::Packet;
use crate::protocol::{region, server_packets};
use std::collections::HashMap;

/// Where players start: (x, y, plane). Estimated — a common starting spot, not from the client.
const SPAWN: (i32, i32, i32) = (3222, 3218, 0);

/// Client variable 1413: every script that reads it (1387 showing a tab's panel, 1534, 2688)
/// returns at once while it is 0, and no script sets it, so only the server can; this server
/// sets it to 1 just before the game's window, whose on-load scripts read it (its own choice
/// of value: the scripts only test ≠ 0).
const GAME_TABS_VARC: (u16, i32) = (1413, 1);

const SKILLS: u8 = 25;
const CONSTITUTION: u8 = 3;
const PRAYER: u8 = 5;

/// A new player's skill: (experience, level). This server's choice: every skill level 1 with no
/// experience, but skill 3 (Constitution, the life points) level 10 with the 1154 it takes.
fn starting_level(skill: u8) -> (i32, u8) {
    if skill == CONSTITUTION {
        (1154, 10)
    } else {
        (0, 1)
    }
}

/// `UPDATE_STAT` for each of a new player's 25 skills.
fn starting_stats() -> impl Iterator<Item = Packet> {
    (0..SKILLS).map(|skill| {
        let (experience, level) = starting_level(skill);
        server_packets::update_stat(skill, experience, level)
    })
}

/// Life and prayer points, full for the starting stats: the orbs show varbit 7198 (varp 1240,
/// bits 1-15) and varbit 9816 (varp 2382, bits 0-14), out of max(base level, 1) × 10 of
/// Constitution (script 2916) and Prayer (script 5255). Only the server sets them.
fn starting_points() -> [Packet; 2] {
    let full = |skill| i32::from(starting_level(skill).1.max(1)) * 10;
    [server_packets::varp(1240, full(CONSTITUTION) << 1), server_packets::varp(2382, full(PRAYER))]
}

/// The game message a player gets on entering the world (this server's own words; the cache's
/// logout tab names the game "Open 727").
pub const WELCOME: &str = "Welcome to Open 727.";

/// The run energy a player starts with (`RUN_ENERGY`, 0-100): full (this server's choice).
const RUN_ENERGY: u8 = 100;

/// Queues everything a player needs to see the world, in the order the client needs it:
/// `REGION` first (the client waits for it before it shows the world), then varc 1413 before
/// the game's window, whose on-load scripts read it.
pub fn first_packets(player: &mut Player, map_keys: &HashMap<u32, Key>) {
    player.send(region::encode(player.index, SPAWN, map_keys));
    player.send(server_packets::varc(GAME_TABS_VARC.0, GAME_TABS_VARC.1));
    player.send(server_packets::run_energy(RUN_ENERGY));
    player.send_all(starting_stats());
    player.send_all(starting_points());
    player.send_all(interfaces::open_window(player.window));
    player.send(server_packets::game_message(0, WELCOME));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::server_prot::ServerProt;

    /// The order matters to the client: `REGION` first, variable 1413 before the window, and
    /// the welcome last (the chat box must be open to show it).
    #[test]
    fn a_player_entering_is_sent_the_world_in_order() {
        let mut player = Player::new(7, "bob".into(), Some(2));
        first_packets(&mut player, &HashMap::new());
        let sent: Vec<ServerProt> = player.take_queued().into_iter().map(|(packet, _)| packet).collect();
        let at = |wanted| sent.iter().position(|p| *p == wanted).unwrap();
        assert_eq!(sent[0], ServerProt::Region);
        assert!(at(ServerProt::ClientSetvarcSmall) < at(ServerProt::IfOpentop));
        assert_eq!(sent.iter().filter(|p| **p == ServerProt::UpdateStat).count(), 25);
        assert_eq!(sent.last(), Some(&ServerProt::GameMessage));
    }
}
