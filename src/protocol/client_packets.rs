//! The payloads of the client → server packets that are ported: the encoder the client uses
//! and, beside it, what the server reads back from the same bytes. What each payload is made of
//! is on its row in `client_prot`.

use super::client_prot::{ClientProt, Packet};

/// `REQUEST_WORLD_LIST`: the checksum of the list the client holds (0 = none).
pub fn request_world_list(checksum: i32) -> Packet {
    (ClientProt::RequestWorldList, checksum.to_be_bytes().to_vec())
}

/// The checksum a `REQUEST_WORLD_LIST` carries.
pub fn world_list_held(payload: &[u8]) -> Option<i32> {
    payload.try_into().ok().map(i32::from_be_bytes)
}

/// `SCREEN_SIZE`: the window mode (`Class158.windowedMode`: 1 fixed, 2 resizable, 3 full
/// screen), the game area's width and height (`BASE_WINDOW_WIDTH`, `BASE_WINDOW_HEIGHT`) as
/// u16s, the anti-aliasing preference (`aPreference_Sub4_8223`).
pub fn screen_size(mode: u8, width: u16, height: u16, anti_aliasing: u8) -> Packet {
    let mut out = vec![mode];
    out.extend(width.to_be_bytes());
    out.extend(height.to_be_bytes());
    out.push(anti_aliasing);
    (ClientProt::ScreenSize, out)
}

/// The window mode a `SCREEN_SIZE` carries.
pub fn screen_size_mode(payload: &[u8]) -> Option<u8> {
    payload.first().copied()
}

/// `IF_OP1` … `IF_OP10` for `option` 1 to 10 (`Class350_Sub3.method12590`): the component's
/// full id as a little-endian int, its item (`containerItemId`, -1 for none) as a u16, its slot
/// as a u16 with 128 added to the low byte (`writeShort128`).
pub fn if_op(option: usize, component: i32, slot: i32, item: i32) -> Packet {
    let mut out = component.to_le_bytes().to_vec();
    out.extend((item as u16).to_be_bytes());
    out.extend([(slot >> 8) as u8, (slot as u8).wrapping_add(128)]);
    (ClientProt::IF_OPS[option - 1], out)
}

/// The component an `IF_OP` was on: interface << 16 | component.
pub fn if_op_component(payload: &[u8]) -> Option<u32> {
    payload.get(..4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn if_op_matches_the_reference() {
        let (packet, payload) = if_op(1, 906 << 16 | 215, -1, -1);
        assert_eq!((packet, &payload[..]), (ClientProt::IfOp1, &[215, 0, 0x8a, 3, 0xff, 0xff, 0xff, 0x7f][..]));
        assert_eq!(if_op_component(&payload), Some(906 << 16 | 215));
        assert_eq!(if_op(10, 0, 0, 0).0, ClientProt::IfOp10);
    }

    #[test]
    fn the_server_reads_back_what_the_client_wrote() {
        assert_eq!(screen_size_mode(&screen_size(2, 1024, 768, 0).1), Some(2));
        assert_eq!(world_list_held(&request_world_list(-7).1), Some(-7));
        assert_eq!(world_list_held(&[1, 2]), None);
    }
}
