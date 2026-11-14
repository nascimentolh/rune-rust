//! The payloads of the server → client packets that are ported, one encoder per packet (or per
//! pair, where the value picks the small or the large one). Each returns the packet with its
//! payload, ready to queue. `REGION` and `WORLD_LIST` have files of their own (`region`,
//! `world_list`). What each payload is made of is on its row in `server_prot`.

use super::server_prot::{Packet, ServerProt};

/// A player variable as `PacketDecoder` reads it: `VARP_SMALL` for a signed byte (`readByte`,
/// `readUnsignedShortLE128`), else `VARP_LARGE` (`readIntV2`, then the id).
pub fn varp(id: u16, value: i32) -> Packet {
    let id_le128 = [(id as u8).wrapping_add(128), (id >> 8) as u8];
    if (-128..=127).contains(&value) {
        (ServerProt::VarpSmall, vec![value as u8, id_le128[0], id_le128[1]])
    } else {
        let v = value.to_be_bytes();
        // `readIntV2`: bytes in the order 2, 1, 4, 3 of the big-endian int.
        (ServerProt::VarpLarge, vec![v[1], v[0], v[3], v[2], id_le128[0], id_le128[1]])
    }
}

/// A client variable: `CLIENT_SETVARC_SMALL` for a signed byte (sent as 128 − value), else
/// `CLIENT_SETVARC_LARGE`.
pub fn varc(id: u16, value: i32) -> Packet {
    let id_le = id.to_le_bytes();
    if (-128..=127).contains(&value) {
        (ServerProt::ClientSetvarcSmall, vec![128u8.wrapping_sub(value as u8), id_le[0], id_le[1]])
    } else {
        let v = value.to_be_bytes();
        (ServerProt::ClientSetvarcLarge, vec![id_le[0], id_le[1], v[1], v[0], v[3], v[2]])
    }
}

/// `GAME_MESSAGE` with no sender: `kind` 0 is a game message.
pub fn game_message(kind: u8, text: &str) -> Packet {
    let mut out = vec![kind, 0, 0, 0, 0, 0];
    out.extend(text.chars().map(crate::cache::text::char_to_cp1252));
    out.push(0);
    (ServerProt::GameMessage, out)
}

/// `UPDATE_STAT`: a skill's experience, id and level.
pub fn update_stat(skill: u8, experience: i32, level: u8) -> Packet {
    let mut out = experience.to_be_bytes().to_vec();
    out.extend([skill, level]);
    (ServerProt::UpdateStat, out)
}

/// `RUN_ENERGY`: 0–100.
pub fn run_energy(energy: u8) -> Packet {
    (ServerProt::RunEnergy, vec![energy])
}

/// `LOGOUT_LOBBY` (back to the lobby) or `LOGOUT_FULL` (to the login screen).
pub fn logout(to_lobby: bool) -> Packet {
    (if to_lobby { ServerProt::LogoutLobby } else { ServerProt::LogoutFull }, Vec::new())
}

/// `IF_OPENTOP` for window `id` (keys and type 0).
pub fn open_top(id: u16) -> Packet {
    let mut out = vec![0u8; 8];
    out.extend([(id >> 8) as u8, (id as u8).wrapping_add(128)]);
    out.push(128);
    out.extend([0u8; 8]);
    (ServerProt::IfOpentop, out)
}

/// `IF_OPENSUB` opening interface `id` in component `parent` (keys 0), `overlay` its type
/// (`readUnsignedByteC`: sent negated).
pub fn open_sub(parent: u32, id: u16, overlay: u8) -> Packet {
    let mut out = vec![0u8; 12];
    out.extend([(id as u8).wrapping_add(128), (id >> 8) as u8]);
    out.extend([0u8; 4]);
    out.extend(parent.to_be_bytes());
    out.push(overlay.wrapping_neg());
    (ServerProt::IfOpensub, out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::packet::Size;

    #[test]
    fn player_variables_small_only_for_a_signed_byte() {
        assert_eq!(varp(2567, 1).0, ServerProt::VarpSmall);
        assert_eq!(varp(5, -1), (ServerProt::VarpSmall, vec![255, 133, 0]));
        // 200 doesn't fit `readByte`: the large packet (id 1240 = 0x4d8, low byte + 128).
        assert_eq!(varp(1240, 200), (ServerProt::VarpLarge, vec![0, 0, 200, 0, 0x58, 4]));
    }

    #[test]
    fn client_variables_small_and_large() {
        assert_eq!(varc(1413, 1), (ServerProt::ClientSetvarcSmall, vec![127, 0x85, 0x05]));
        assert_eq!(varc(1413, -1), (ServerProt::ClientSetvarcSmall, vec![129, 0x85, 0x05]));
        assert_eq!(varc(2, 0x0102_0304), (ServerProt::ClientSetvarcLarge, vec![2, 0, 2, 1, 4, 3]));
    }

    /// Every encoder of a fixed-size packet fills exactly that size.
    #[test]
    fn fixed_payloads_have_the_reference_size() {
        let packets =
            [varp(1, 1), varp(1, 999), varc(1, 1), varc(1, 999), update_stat(3, 1154, 10), run_energy(100), logout(true), logout(false)];
        for (packet, payload) in packets.into_iter().chain([open_top(746), open_sub(746 << 16 | 22, 752, 1)]) {
            assert_eq!(packet.size(), Size::Fixed(payload.len()), "{}", packet.name());
        }
        // 746 = 0x02ea: the high byte, then the low byte + 128.
        assert_eq!(open_top(746).1[8..10], [2, 0xea_u8.wrapping_add(128)]);
        assert_eq!(update_stat(3, 1154, 10).1, vec![0, 0, 4, 0x82, 3, 10]);
    }
}
