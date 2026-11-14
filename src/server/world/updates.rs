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
//! Ported so far: the players' own entry (a step, their appearance when it changed, their
//! movement type when it has to be sent). Other players coming into view are next; until then a
//! client's first list holds only its own player, and every other index is skipped.

use super::player::Player;
use crate::protocol::bits::BitWriter;
use crate::protocol::huffman::Huffman;
use crate::protocol::player_update::{write_local, write_skip, Masks, Step, WALKING};
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

/// What a client is told about a player it has: their step and, when something in it changed,
/// a mask block.
#[derive(Clone)]
struct Local {
    step: Step,
    masks: Option<Vec<u8>>,
}

/// `PLAYER_UPDATE`'s payload for `viewer`, and their `View` moved on to what the client will
/// hold after reading it. `update(index)` is what changed about a player the client has, or
/// `None` when nothing did.
fn encode(view: &mut View, update: impl Fn(u16) -> Option<Local>) -> Vec<u8> {
    let outside: Vec<u16> = (1..INDICES).filter(|&i| !view.has(i)).collect();
    let mut skipped = vec![false; usize::from(INDICES)];
    let (mut bits, mut masks) = (Vec::new(), Vec::<u8>::new());
    // `decodeUpdate`'s groups, each from a fresh byte: the players it has that weren't skipped,
    // those that were, then the others that were, and those that weren't.
    for (list, was_skipped, has) in
        [(&view.local, false, true), (&view.local, true, true), (&outside, true, false), (&outside, false, false)]
    {
        let group: Vec<u16> = list.iter().copied().filter(|&i| view.skipped[usize::from(i)] == was_skipped).collect();
        let blocks: Vec<Option<Local>> = group.iter().map(|&i| if has { update(i) } else { None }).collect();
        let mut w = BitWriter::default();
        let mut at = 0;
        while at < group.len() {
            if let Some(local) = &blocks[at] {
                // `method1806`: an update, whether a mask block follows, the step.
                write_local(&mut w, local.masks.is_some(), local.step);
                masks.extend(local.masks.iter().flatten());
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

/// What changed about a player this tick, or `None`. Their appearance while it is marked
/// changed, and their movement type (walking: running isn't ported) while that is, as the
/// reference server sends both to a new player (`updateMovementType` starts true).
fn local(player: &Player) -> Option<Local> {
    let masks = Masks {
        appearance: player.appearance_changed.then(|| player.appearance.encode()),
        movement_type: player.movement_type_changed.then_some(WALKING),
    };
    let masks = (masks != Masks::default()).then(|| masks.encode());
    (player.step != Step::Stay || masks.is_some()).then_some(Local { step: player.step, masks })
}

/// Stage: `PLAYER_UPDATE` for every player, in index order, then the marks of what was sent
/// are cleared.
pub fn send_players(players: &mut [&mut Player]) {
    let changed: Vec<(u16, Local)> = players.iter().filter_map(|p| Some((p.index, local(p)?))).collect();
    for player in players.iter_mut() {
        let payload = encode(&mut player.view, |i| changed.iter().find(|(c, _)| *c == i).map(|(_, l)| l.clone()));
        player.send(server_packets::player_update(payload));
    }
    for player in players.iter_mut() {
        player.appearance_changed = false;
        player.movement_type_changed = false;
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
    use crate::protocol::player_update::{read_local, read_skip, read_step};
    use crate::protocol::server_prot::ServerProt;

    /// Reads a payload the way `decodeUpdate` does for a client whose only player is itself:
    /// `None` when it was skipped, else its step and its masks. An empty group takes no bytes,
    /// so its own entry is the first group read and the 2046 other indices the second.
    fn read_as_client(payload: &[u8]) -> Option<(Step, Option<Masks>)> {
        let mut r = BitReader::new(payload, 0);
        let entry = if r.read(1) == Some(1) {
            let (masks, kind) = read_local(&mut r).unwrap();
            Some((if kind == 0 { Step::Stay } else { read_step(&mut r, kind).unwrap() }, masks))
        } else {
            assert_eq!(read_skip(&mut r), Some(0));
            None
        };
        let mut r = BitReader::new(payload, r.finish());
        assert_eq!((r.read(1), read_skip(&mut r)), (Some(0), Some(2045)));
        let mut at = r.finish();
        let entry = entry.map(|(step, masks)| (step, masks.then(|| Masks::read(payload, &mut at).unwrap())));
        assert_eq!(at, payload.len(), "the client throws unless the packet is read to its end");
        entry
    }

    #[test]
    fn the_first_update_sends_the_appearance_and_later_ones_skip() {
        let mut player = Player::new(5, "bob".into(), Some(1));
        send_players(&mut [&mut player]);
        let sent = player.take_queued();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, ServerProt::PlayerUpdate);
        let (step, masks) = read_as_client(&sent[0].1).unwrap();
        let masks = masks.unwrap();
        assert_eq!((step, masks.movement_type), (Step::Stay, Some(WALKING)));
        assert_eq!(masks.appearance, Some(player.appearance.encode()));
        assert!(!player.appearance_changed);
        // Then they are skipped (group 1, and from then on group 2, as are the others).
        for _ in 0..2 {
            send_players(&mut [&mut player]);
            assert_eq!(read_as_client(&player.take_queued().remove(0).1), None);
        }
        assert!(player.view.skipped.iter().skip(1).all(|&s| s));
        // A step goes without a mask block.
        player.step = Step::Walk(6);
        send_players(&mut [&mut player]);
        assert_eq!(read_as_client(&player.take_queued().remove(0).1), Some((Step::Walk(6), None)));
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
