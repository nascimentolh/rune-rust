//! Client packets about chat.

use crate::protocol::client_packets;
use crate::protocol::huffman::Huffman;
use crate::server::world::player::{Player, PublicChat};

/// `CHAT_TYPE`: where the next `CHAT` goes. Kept as the reference server keeps it
/// (`ChatTypeHandler`: `player.chatType = packet.type`), since `CHAT` itself doesn't say.
pub fn chat_type(player: &mut Player, payload: &[u8]) {
    if let Some(kind) = client_packets::chat_type_value(payload) {
        player.chat_type = kind;
    }
}

/// `CHAT`: with the chat type public (0), the line is kept for the tick's chat stage with its
/// effects as `MESSAGE_PUBLIC` carries them (colour << 8 | effect). As the reference server's
/// `ChatHandler`: a blank line is dropped, and so is one starting "::" or ";;" (a command there;
/// this server has none yet). Friends and clan chat aren't ported. Without a Huffman table the
/// text can't be read.
pub fn chat(player: &mut Player, payload: &[u8], huffman: Option<&Huffman>) {
    let Some((colour, effect, text)) = huffman.and_then(|h| client_packets::chat_message(payload, h)) else { return };
    if text.starts_with("::") || text.starts_with(";;") || text.trim().is_empty() || player.chat_type != 0 {
        return;
    }
    player.public_chat = Some(PublicChat { effects: u16::from(colour) << 8 | u16::from(effect), text });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_chat_is_kept_for_the_chat_stage() {
        let mut lengths = vec![0u8; 256];
        for b in b"hi: " {
            lengths[usize::from(*b)] = 2;
        }
        let huffman = Huffman::new(&lengths);
        let mut player = Player::new(1, "bob".into(), Some(1));
        let (_, said) = client_packets::chat(2, 1, "hi", &huffman).unwrap();
        chat(&mut player, &said, Some(&huffman));
        assert_eq!(player.public_chat, Some(PublicChat { effects: 0x0201, text: "hi".into() }));
        // Blank, a command, or another chat type: nothing.
        for (kind, text) in [(0, "  "), (0, "::hi"), (1, "hi")] {
            player.public_chat = None;
            chat_type(&mut player, &client_packets::chat_type(kind).1);
            chat(&mut player, &client_packets::chat(0, 0, text, &huffman).unwrap().1, Some(&huffman));
            assert_eq!(player.public_chat, None);
        }
    }
}
