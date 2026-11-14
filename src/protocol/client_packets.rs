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

/// `WALK` (`CutsceneArea.method12078` with 0): whether the run key is held (`method1465`), then
/// the destination's x (u16) and y (`writeShortLE`), in tiles of the whole map.
pub fn walk(force_run: bool, x: u16, y: u16) -> Packet {
    let mut out = vec![u8::from(force_run)];
    out.extend(x.to_be_bytes());
    out.extend(y.to_le_bytes());
    (ClientProt::Walk, out)
}

/// The destination a `WALK` asks for: (x, y).
pub fn walk_destination(payload: &[u8]) -> Option<(i32, i32)> {
    let x = u16::from_be_bytes(payload.get(1..3)?.try_into().ok()?);
    let y = u16::from_le_bytes(payload.get(3..5)?.try_into().ok()?);
    Some((i32::from(x), i32::from(y)))
}

/// The window mode a `SCREEN_SIZE` carries.
pub fn screen_size_mode(payload: &[u8]) -> Option<u8> {
    payload.first().copied()
}

/// `CHAT_TYPE`: the chat the next message is for, one byte.
pub fn chat_type(kind: u8) -> Packet {
    (ClientProt::ChatType, vec![kind])
}

/// What a `CHAT_TYPE` holds.
pub fn chat_type_value(payload: &[u8]) -> Option<u8> {
    payload.first().copied()
}

/// `CHAT`: `colour` and `effect`, then `text` as `ByteBuf.writeHuffmanString`: its length in
/// characters (`method13076`: a byte below 128, else a u16 + 32768; at most 32767), then its
/// Windows-1252 bytes (`LinkedNodeList.method7885`) compressed. `None` when the text is too
/// long or has a character the table can't code (the reference throws).
pub fn chat(colour: u8, effect: u8, text: &str, huffman: &super::huffman::Huffman) -> Option<Packet> {
    let mut out = vec![colour, effect];
    out.extend(huffman.write_text(text)?);
    Some((ClientProt::Chat, out))
}

/// What a `CHAT` holds: (colour, effect, text), the text decompressed with the same table.
pub fn chat_message(payload: &[u8], huffman: &super::huffman::Huffman) -> Option<(u8, u8, String)> {
    Some((*payload.first()?, *payload.get(1)?, huffman.read_text(payload.get(2..)?)?))
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
    fn walk_has_the_run_key_then_x_then_y_little_endian() {
        let (packet, payload) = walk(false, 3225, 3218);
        assert_eq!((packet, payload.clone()), (ClientProt::Walk, vec![0, 0x0c, 0x99, 0x92, 0x0c]));
        assert_eq!(walk_destination(&payload), Some((3225, 3218)));
        assert_eq!(walk_destination(&payload[..4]), None);
    }

    #[test]
    fn chat_carries_its_length_then_the_compressed_text() {
        let mut lengths = vec![0u8; 256];
        lengths[usize::from(b'h')] = 1;
        lengths[usize::from(b'i')] = 1;
        let huffman = super::super::huffman::Huffman::new(&lengths);
        // h = 0, i = 1: "hi" → 01 padded → 0x40.
        assert_eq!(chat(2, 1, "hi", &huffman), Some((ClientProt::Chat, vec![2, 1, 2, 0x40])));
        assert_eq!(chat_message(&[2, 1, 2, 0x40], &huffman), Some((2, 1, "hi".into())));
        assert_eq!(chat(0, 0, "x", &huffman), None);
    }

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
