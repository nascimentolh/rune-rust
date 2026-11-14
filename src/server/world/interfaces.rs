//! The game's window and what is opened in it: which interface goes where. The client opens
//! whatever it is sent, so all of this is the server's choice, made from what is in the cache;
//! each choice says how it was made.

use crate::protocol::server_packets;
use crate::protocol::server_prot::Packet;

/// The fixed-size and the resizable game window.
const FIXED: u16 = 548;
const RESIZABLE: u16 = 746;

/// The game's window for the client's window mode (1 fixed, 2 resizable, 3 full screen):
/// interface 548 at the fixed size, 746 otherwise, as 727 servers open it with `IF_OPENTOP`.
pub fn game_window(mode: Option<u8>) -> u16 {
    match mode {
        Some(mode) if mode >= 2 => RESIZABLE,
        _ => FIXED,
    }
}

/// The side panels, by tab: the tab buttons pass their tab to script 1303 (548:112-119 tabs
/// 0-7, 548:83-90 tabs 8-15), and script 1387 shows 548:(176 + tab) or 746:(112 + tab) for it.
/// Which interface goes in each is the server's choice; these are the reference server's
/// (`InterfaceManager.Sub.TAB_*`, the same slots): combat styles, task system, stats, quests,
/// inventory, worn equipment, prayer, magic, follower, friends, friends chat, clan chat,
/// options, emotes, music, notes. Tab 8 (the follower, 662) opens only with a familiar or a pet
/// there, so it stays empty.
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

/// The logout tab: the Exit button (548:159, 746:195) passes tab 99, which script 1387 shows
/// in 548:194 or 746:130; this server opens interface 182 there (its on-load script 1104 names
/// its buttons "Exit to Lobby", 182:6, and "Exit to Login", 182:13).
const LOGOUT_PANEL: u16 = 182;
pub const EXIT_TO_LOBBY: u32 = (LOGOUT_PANEL as u32) << 16 | 6;
pub const EXIT_TO_LOGIN: u32 = (LOGOUT_PANEL as u32) << 16 | 13;

/// The orbs by the minimap: (interface, 548 slot, 746 slot). The 746 slots carry their own
/// tooltips in the cache (script 4102: "Your current Life Points.", "…Prayer Points…", "…run
/// energy…", "…Summoning energy…"); the 548 slots are 57 × 34 in the same order, and script
/// 2756 pairs 548:162 with 746:198 (run). The interfaces are the reference server's
/// (`InterfaceManager.Sub.ORB_*`): 748 life ("Use Cure"), 749 prayer, 750 run, 747 summoning.
const ORBS: [(u16, u32, u32); 4] = [(748, 160, 196), (749, 161, 197), (750, 162, 198), (747, 164, 199)];

/// The chat: (interface, 548 slot, 746 slot), opened in this order. 752 (the chat frame) goes
/// in the chat area, 548:168 and 746:22, which script 84 treats as one
/// (`IF_SETNOCLICKTHROUGH` on both); the filter bar 751 in 548:53 and 746:23; the chat box 137
/// in 752:9, the slot script 1364 shows when nothing is open in 752:10-13 (dialogues and
/// prompts). Slots and order are the reference server's (`InterfaceManager.Sub.CHATBOX`, then
/// `CHATBAR_SETTINGS`, whose opening sends 137 to `CHATBOX_LINES`, 752:9). 137 must come after
/// 751: its scripts size 751 to its own width (script 178: `IF_GETWIDTH`, then `IF_SETSIZE`),
/// which is 0 until 751 is open and laid out.
const CHAT: [(u16, u32, u32); 3] =
    [(752, 548 << 16 | 168, 746 << 16 | 22), (751, 548 << 16 | 53, 746 << 16 | 23), (137, 752 << 16 | 9, 752 << 16 | 9)];

/// The packets that open `window` (548 or 746) and everything in it: `IF_OPENTOP`, then an
/// `IF_OPENSUB` for each side panel, the logout tab, the orbs and the chat, as overlays.
pub fn open_window(window: u16) -> Vec<Packet> {
    let resizable = window == RESIZABLE;
    let (first_slot, logout_slot) = if resizable { (112, 130) } else { (176, 194) };
    let orbs = ORBS.iter().map(|&(id, fixed, other)| (if resizable { other } else { fixed }, id));
    let chat = CHAT.iter().map(|&(id, fixed, other)| (if resizable { other } else { fixed }, id));
    let panels = TAB_PANELS
        .iter()
        .zip(first_slot..)
        .filter_map(|(panel, slot)| panel.map(|id| (slot, id)))
        .chain([(logout_slot, LOGOUT_PANEL)])
        .chain(orbs)
        .map(|(slot, id)| ((window as u32) << 16 | slot, id))
        .chain(chat)
        .map(|(parent, id)| server_packets::open_sub(parent, id, 1));
    std::iter::once(server_packets::open_top(window)).chain(panels).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::server_prot::ServerProt;

    /// The window, then its 15 side panels, the logout tab, 4 orbs and 3 chat interfaces.
    #[test]
    fn a_window_opens_with_everything_in_it() {
        for (mode, window) in [(None, 548), (Some(1), 548), (Some(2), 746), (Some(3), 746)] {
            assert_eq!(game_window(mode), window);
            let packets = open_window(window);
            assert_eq!(packets[0], server_packets::open_top(window));
            assert_eq!(packets.len(), 1 + 15 + 1 + 4 + 3);
            assert!(packets[1..].iter().all(|(packet, _)| *packet == ServerProt::IfOpensub));
        }
        // The inventory (679) in tab 4's slot of the resizable window: 746:116.
        assert!(open_window(746).contains(&server_packets::open_sub(746 << 16 | 116, 679, 1)));
    }
}
