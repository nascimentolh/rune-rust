//! The tick's update stages: what each player's client is told about the players
//! (`PLAYER_UPDATE`, every tick) and what they said (`MESSAGE_PUBLIC`).
//!
//! The client keeps two lists (`Class197`): the players it has (`PLAYER_INDICES`) and every
//! other index (`anIntArray2426`), and for each index whether the last update skipped it
//! (`aByteArray2424`). `REGION` starts them (`method3203`): only the player themselves is in
//! the first list, nothing was skipped. Every `PLAYER_UPDATE` walks both lists in four groups
//! (`PlayerUpdate.decodeUpdate`), so the server has to hold the same lists for each client and
//! write a field for every index in the order the client reads them. `View` is that copy.
//!
//! Ported so far: the players' own entry (staying where they are; their appearance when it
//! changed). Other players coming into view, and movement, are next; until then a client's
//! first list holds only its own player, and every other index is skipped.

use super::player::Player;
use crate::protocol::bits::BitWriter;
use crate::protocol::huffman::Huffman;
use crate::protocol::player_update::{appearance_mask, write_local_stay, write_skip};
use crate::protocol::server_packets;

/// The highest index + 1 the client's lists cover (`for (i = 1; i < 2048; i++)`).
const INDICES: u16 = 2048;

/// One client's player lists, as the server keeps them in step.
pub struct View {
    /// The players the client has (`PLAYER_INDICES`), in its order.
    local: Vec<u16>,
    /// Per index: the last update skipped it (`aByteArray2424` bit 0).
    skipped: Vec<bool>,
}

impl View {
    /// The lists as `REGION` leaves them (`Class197.method3203`): only `me`, nothing skipped.
    pub fn new(me: u16) -> Self {
        View { local: vec![me], skipped: vec![false; usize::from(INDICES)] }
    }

    /// Whether the client has player `index`.
    pub fn has(&self, index: u16) -> bool {
        self.local.contains(&index)
    }
}

/// `PLAYER_UPDATE`'s payload for `viewer`, and their `View` moved on to what the client will
/// hold after reading it. `update(index)` is the mask block for a player the client has, or
/// `None` when nothing about them changed.
fn encode(view: &mut View, update: impl Fn(u16) -> Option<Vec<u8>>) -> Vec<u8> {
    let outside: Vec<u16> = (1..INDICES).filter(|&i| !view.has(i)).collect();
    let mut skipped = vec![false; usize::from(INDICES)];
    let (mut bits, mut masks) = (Vec::new(), Vec::<u8>::new());
    // `decodeUpdate`'s groups, each from a fresh byte: the players it has that weren't skipped,
    // those that were, then the others that were, and those that weren't.
    for (list, was_skipped, has) in
        [(&view.local, false, true), (&view.local, true, true), (&outside, true, false), (&outside, false, false)]
    {
        let group: Vec<u16> = list.iter().copied().filter(|&i| view.skipped[usize::from(i)] == was_skipped).collect();
        let blocks: Vec<Option<Vec<u8>>> = group.iter().map(|&i| if has { update(i) } else { None }).collect();
        let mut w = BitWriter::default();
        let mut at = 0;
        while at < group.len() {
            if let Some(block) = &blocks[at] {
                // `method1806`: an update, a mask block follows, staying where they are.
                write_local_stay(&mut w, true);
                masks.extend(block);
                at += 1;
                continue;
            }
            // A 0 bit, then how many more after this one are skipped too (`decodeSkip`).
            let run = blocks[at..].iter().take_while(|b| b.is_none()).count();
            w.write(1, 0);
            write_skip(&mut w, run as u32 - 1);
            for &i in &group[at..at + run] {
                skipped[usize::from(i)] = true;
            }
            at += run;
        }
        bits.extend(w.bytes);
    }
    // After the groups the client shifts each index's flags down: skipped now = skipped last.
    view.skipped = skipped;
    bits.extend(masks);
    bits
}

/// Stage: `PLAYER_UPDATE` for every player, in index order. A player's appearance goes to
/// every client that has them while it is marked changed, then the mark is cleared.
pub fn send_players(players: &mut [&mut Player]) {
    let changed: Vec<(u16, Vec<u8>)> =
        players.iter().filter(|p| p.appearance_changed).map(|p| (p.index, appearance_mask(&p.appearance.encode()))).collect();
    for player in players.iter_mut() {
        let payload = encode(&mut player.view, |i| changed.iter().find(|(c, _)| *c == i).map(|(_, b)| b.clone()));
        player.send(server_packets::player_update(payload));
    }
    for player in players.iter_mut() {
        player.appearance_changed = false;
    }
}

/// Stage: each line of public chat said this tick goes to every client that has the speaker,
/// the speaker's own included, as `MESSAGE_PUBLIC` with icon 0 (no crown: this server has no
/// staff). The reference server does the same (`Player.sendPublicChatMessage`: every nearby
/// player whose local players hold the speaker). Without a Huffman table nothing can be sent.
pub fn send_public_chat(players: &mut [&mut Player], huffman: Option<&Huffman>) {
    let said: Vec<(u16, super::player::PublicChat)> = players.iter_mut().filter_map(|p| Some((p.index, p.public_chat.take()?))).collect();
    let Some(huffman) = huffman else { return };
    for player in players.iter_mut() {
        for (speaker, chat) in &said {
            if player.view.has(*speaker) {
                if let Some(packet) = server_packets::message_public(*speaker, chat.effects, 0, &chat.text, huffman) {
                    player.send(packet);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::bits::BitReader;
    use crate::protocol::player_update::{read_local, read_masks, read_skip};
    use crate::protocol::server_prot::ServerProt;

    /// Reads a payload the way `decodeUpdate` does for a client whose only player is itself:
    /// (it was updated with a mask block, it was skipped). An empty group takes no bytes, so
    /// its own entry is the first group read and the 2046 other indices the second.
    fn read_as_client(payload: &[u8]) -> (bool, bool) {
        let mut r = BitReader::new(payload, 0);
        let updated = r.read(1) == Some(1);
        if updated {
            assert_eq!(read_local(&mut r), Some((true, 0)));
        } else {
            assert_eq!(read_skip(&mut r), Some(0));
        }
        let mut r = BitReader::new(payload, r.finish());
        assert_eq!((r.read(1), read_skip(&mut r)), (Some(0), Some(2045)));
        let mut at = r.finish();
        if updated {
            assert!(read_masks(payload, &mut at).unwrap().is_some());
        }
        assert_eq!(at, payload.len(), "the client throws unless the packet is read to its end");
        (updated, !updated)
    }

    #[test]
    fn the_first_update_sends_the_appearance_and_later_ones_skip() {
        let mut player = Player::new(5, "bob".into(), Some(1));
        send_players(&mut [&mut player]);
        let sent = player.take_queued();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, ServerProt::PlayerUpdate);
        assert_eq!(read_as_client(&sent[0].1), (true, false));
        assert!(!player.appearance_changed);
        // Then they are skipped (group 1, and from then on group 2, as are the others).
        for _ in 0..2 {
            send_players(&mut [&mut player]);
            assert_eq!(read_as_client(&player.take_queued().remove(0).1), (false, true));
        }
        assert!(player.view.skipped.iter().skip(1).all(|&s| s));
    }

    #[test]
    fn public_chat_goes_to_those_who_have_the_speaker() {
        let mut lengths = vec![0u8; 256];
        for b in b"hi" {
            lengths[usize::from(*b)] = 1;
        }
        let huffman = Huffman::new(&lengths);
        let mut bob = Player::new(1, "bob".into(), Some(1));
        let mut ann = Player::new(2, "ann".into(), Some(1));
        bob.public_chat = Some(super::super::player::PublicChat { effects: 0x0102, text: "hi".into() });
        send_public_chat(&mut [&mut bob, &mut ann], Some(&huffman));
        let to_bob = bob.take_queued();
        assert_eq!(to_bob.len(), 1);
        assert_eq!(server_packets::public_message(&to_bob[0].1, &huffman), Some((1, 0x0102, 0, "hi".into())));
        // Ann's client doesn't have bob (other players coming into view aren't ported).
        assert!(ann.take_queued().is_empty());
        assert_eq!(bob.public_chat, None);
    }
}
