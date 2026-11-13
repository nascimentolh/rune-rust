//! The first game packet of a world login: `REGION` (server opcode 85, size −2 = a u16 length),
//! which the client reads right after the login reply (`Login.java`, login stages 167 → 173 →
//! 156) before it shows the world.
//!
//! Its payload, as the client reads it:
//! - `Class197.method3203` (player positions, in bits): 30 bits for this player
//!   (plane << 28 | x << 14 | y), then 18 bits for each other index 1..2047
//!   (plane << 16 | region x << 8 | region y; 0 = nobody known yet);
//! - `MapRegion.loadMapSceneNormal`: map size (0 = 104 tiles), the centre chunk x and y (u16),
//!   a "force reload" byte, then four XTEA key ints for every map region around the centre:
//!   region x from (chunk x − 6) / 8 to (chunk x + 6) / 8, and within it region y the same way.
//!
//! Where a new player starts is this server's choice (`SPAWN`).

use crate::cache::xtea::Key;
use std::collections::HashMap;

/// Where players start: (x, y, plane). Estimated — a common starting spot, not from the client.
pub const SPAWN: (i32, i32, i32) = (3222, 3218, 0);

/// The game's window for the client's display mode (the login's window mode: 1 fixed, 2
/// resizable, 3 full screen): interface 548 at the fixed size, 746 otherwise, as 727 servers
/// open it with `IF_OPENTOP`. (The server's choice: the client opens whatever it is sent.)
pub fn game_window(display: Option<(u8, u16, u16)>) -> u16 {
    match display {
        Some((mode, _, _)) if mode >= 2 => 746,
        _ => 548,
    }
}

/// The side panels, by tab: the tab buttons pass their tab to script 1303 (548:112-119 tabs
/// 0-7, 548:83-90 tabs 8-15), and script 1387 shows 548:(176 + tab) or 746:(112 + tab) for it.
/// Which interface goes in each is the server's choice: these are recognised by their text in
/// the cache (combat styles, task system, stats, quests, inventory, worn equipment, prayer,
/// magic, —, friends, friends chat, clan chat, options, emotes, music, notes). Tab 8
/// (Resources) stays empty.
const TAB_PANELS: [Option<u16>; 16] = [
    Some(884),
    Some(1056),
    Some(320),
    Some(190),
    Some(679),
    Some(387),
    Some(271),
    Some(192),
    None,
    Some(550),
    Some(1109),
    Some(1110),
    Some(261),
    Some(590),
    Some(187),
    Some(34),
];

/// Client variable 1413: every script that reads it (1387 showing a tab's panel, 1534, 2688)
/// returns at once while it is 0, and no script sets it, so only the server can; this server
/// sets it to 1 just before the game's window, whose on-load scripts read it (its own choice
/// of value: the scripts only test ≠ 0).
pub const GAME_TABS_VARC: (u16, i32) = (1413, 1);

/// The logout tab: the Exit button (548:159, 746:195) passes tab 99, which script 1387 shows
/// in 548:194 or 746:130; this server opens interface 182 there (its on-load script 1104 names
/// its buttons "Exit to Lobby", 182:6, and "Exit to Login", 182:13).
const LOGOUT_PANEL: u16 = 182;
pub const EXIT_TO_LOBBY: u32 = (LOGOUT_PANEL as u32) << 16 | 6;
pub const EXIT_TO_LOGIN: u32 = (LOGOUT_PANEL as u32) << 16 | 13;

/// The orbs by the minimap: (interface, 548 slot, 746 slot). The 746 slots carry their own
/// tooltips in the cache (script 4102: "Your current Life Points.", "…Prayer Points…", "…run
/// energy…", "…Summoning energy…"); the 548 slots are 57 × 34 in the same order, and script
/// 2756 pairs 548:162 with 746:198 (run). The interfaces are the server's choice, recognised
/// by their options: 748 "Use Cure", 749 quick prayers, 750 run mode, 747 the follower.
const ORBS: [(u16, u32, u32); 4] = [(748, 160, 196), (749, 161, 197), (750, 162, 198), (747, 164, 199)];

/// The `IF_OPENSUB` payloads that open the side panels, the logout tab and the orbs in
/// `window` (548 or 746), as overlays.
pub fn side_panels(window: u16) -> Vec<Vec<u8>> {
    let (first_slot, logout_slot) = if window == 746 { (112, 130) } else { (176, 194) };
    let orbs = ORBS.iter().map(|&(id, fixed, resizable)| (if window == 746 { resizable } else { fixed }, id));
    TAB_PANELS
        .iter()
        .zip(first_slot..)
        .filter_map(|(panel, slot)| panel.map(|id| (slot, id)))
        .chain([(logout_slot, LOGOUT_PANEL)])
        .chain(orbs)
        .map(|(slot, id)| super::server_prot::open_sub((window as u32) << 16 | slot, id, 1))
        .collect()
}
/// `MapSize.SIZE_104.index`, and its tiles >> 4 (`sizeX >> 4`).
const MAP_SIZE: u8 = 0;
const HALF_SPAN: i32 = 104 >> 4;

/// MSB-first bit writer (`ByteBuf.Bit.writeBits`).
#[derive(Default)]
struct Bits {
    bytes: Vec<u8>,
    at: usize,
}

impl Bits {
    fn write(&mut self, count: u32, value: u32) {
        for i in (0..count).rev() {
            if self.at.is_multiple_of(8) {
                self.bytes.push(0);
            }
            if value >> i & 1 == 1 {
                let last = self.bytes.len() - 1;
                self.bytes[last] |= 0x80 >> (self.at % 8);
            }
            self.at += 1;
        }
    }
}

/// The map regions `loadMapSceneNormal` asks keys for, in its order.
pub fn regions(chunk_x: i32, chunk_y: i32) -> Vec<u32> {
    let mut out = Vec::new();
    for rx in (chunk_x - HALF_SPAN) / 8..=(chunk_x + HALF_SPAN) / 8 {
        for ry in (chunk_y - HALF_SPAN) / 8..=(chunk_y + HALF_SPAN) / 8 {
            out.push((rx << 8 | ry) as u32);
        }
    }
    out
}

/// The `REGION` payload for player `index` at `at`, with the map keys by region id.
pub fn region_payload(index: u16, at: (i32, i32, i32), keys: &HashMap<u32, Key>) -> Vec<u8> {
    let (x, y, plane) = at;
    let mut bits = Bits::default();
    bits.write(30, (plane << 28 | x << 14 | y) as u32);
    for other in 1..2048u16 {
        if other != index {
            bits.write(18, 0);
        }
    }
    let mut out = bits.bytes;
    let (cx, cy) = (x >> 3, y >> 3);
    out.push(MAP_SIZE);
    out.extend((cx as u16).to_be_bytes());
    out.extend((cy as u16).to_be_bytes());
    out.push(1);
    for region in regions(cx, cy) {
        for k in keys.get(&region).copied().unwrap_or_default() {
            out.extend(k.to_be_bytes());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_are_written_high_bit_first() {
        let mut b = Bits::default();
        b.write(3, 0b101);
        b.write(6, 0b111111);
        assert_eq!(b.bytes, vec![0b1011_1111, 0b1000_0000]);
    }

    #[test]
    fn the_payload_holds_the_spawn_and_one_key_per_region() {
        let p = region_payload(1, SPAWN, &HashMap::new());
        let bits_len = (30 + 2046 * 18_usize).div_ceil(8);
        let first = u32::from_be_bytes([p[0], p[1], p[2], p[3]]) >> 2;
        assert_eq!(first, (SPAWN.0 << 14 | SPAWN.1) as u32);
        let n = regions(SPAWN.0 >> 3, SPAWN.1 >> 3).len();
        assert_eq!(p.len(), bits_len + 6 + n * 16);
    }
}
