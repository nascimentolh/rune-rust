//! The players the client knows (`client.players`, `Class197`'s lists) and `PLAYER_UPDATE`
//! read into them (`PlayerUpdate.decode`).
//!
//! `REGION` makes the client's own player (`Class197.method3203`): the first list
//! (`PLAYER_INDICES`) holds only them, the second (`anIntArray2426`) every other index, and no
//! index was skipped (`aByteArray2424` all 0). Every `PLAYER_UPDATE` then walks the lists in
//! four groups, each from a fresh byte; see `protocol::player_update`.
//!
//! Ported: a player the client has staying where they are, with the appearance mask. A path
//! that isn't ported (movement, a player leaving or coming into view, another mask) stops the
//! packet with an error, where the reference would carry on.

use crate::protocol::bits::BitReader;
use crate::protocol::player_update::{read_local, read_masks, read_skip, Appearance};

/// The highest index + 1 (`client.players` has 2048 entries).
const INDICES: usize = 2048;

/// A player entity, as far as it's ported: the appearance once the server sent it
/// (`displayName` and `model` are set by `decodeAppearance`).
#[derive(Default)]
pub struct Player {
    pub appearance: Option<Appearance>,
}

pub struct Players {
    /// `client.myPlayerIndex`.
    me: u16,
    /// `client.players`.
    by_index: Vec<Option<Player>>,
    /// `PLAYER_INDICES` and `anIntArray2426`.
    local: Vec<u16>,
    outside: Vec<u16>,
    /// `aByteArray2424`: bit 0 = skipped by the last update, bit 1 = skipped by this one.
    flags: Vec<u8>,
}

impl Players {
    /// The lists as `REGION` leaves them (`method3203`). Its other players' positions (18 bits
    /// each) aren't kept yet: nothing reads them until players come into view.
    pub fn new(me: u16) -> Self {
        let mut by_index: Vec<Option<Player>> = (0..INDICES).map(|_| None).collect();
        by_index[usize::from(me)] = Some(Player::default());
        Players { me, by_index, local: vec![me], outside: (1..INDICES as u16).filter(|&i| i != me).collect(), flags: vec![0; INDICES] }
    }

    /// `VertexNormal.MY_PLAYER`.
    pub fn me(&self) -> Option<&Player> {
        self.get(self.me)
    }

    /// `client.players[index]` (the own player for `myPlayerIndex`).
    pub fn get(&self, index: u16) -> Option<&Player> {
        self.by_index.get(usize::from(index))?.as_ref()
    }

    /// `PlayerUpdate.decode`: the four groups (`decodeUpdate`), then each mask block in the
    /// order the updates asked for them (`decodeMasks`); the packet must be read to its end.
    /// `hidden` is the cache's equipment slot table, for the appearances.
    pub fn update(&mut self, payload: &[u8], hidden: &[u8]) -> Result<(), String> {
        let mut with_masks = Vec::new();
        let mut at = 0;
        let mut skip = 0;
        // Players the client has, not skipped last time, then skipped; the others, skipped last
        // time, then not.
        for (local, was_skipped) in [(true, false), (true, true), (false, true), (false, false)] {
            // Only bit 1 changes during a group, so its members can be picked first.
            let list = if local { &self.local } else { &self.outside };
            let group: Vec<u16> = list.iter().copied().filter(|&i| (self.flags[usize::from(i)] & 1 != 0) == was_skipped).collect();
            let mut r = BitReader::new(payload, at);
            for i in group {
                let flags = &mut self.flags[usize::from(i)];
                if skip > 0 {
                    skip -= 1;
                    *flags |= 2;
                } else if r.read(1).ok_or("PLAYER_UPDATE runs out")? == 0 {
                    skip = read_skip(&mut r).ok_or("PLAYER_UPDATE runs out")?;
                    *flags |= 2;
                } else if local {
                    // `NewsItem.method1806`.
                    let (masks, kind) = read_local(&mut r).ok_or("PLAYER_UPDATE runs out")?;
                    if masks {
                        with_masks.push(i);
                    }
                    match (kind, masks) {
                        (0, true) => {}
                        (0, false) => return Err(format!("PLAYER_UPDATE: player {i} leaving view isn't ported")),
                        _ => return Err(format!("PLAYER_UPDATE: player {i} moving (type {kind}) isn't ported")),
                    }
                } else {
                    return Err(format!("PLAYER_UPDATE: player {i} coming into view isn't ported"));
                }
            }
            at = r.finish();
            // `decodeUpdate` throws when a skip runs past its group.
            if skip != 0 {
                return Err("PLAYER_UPDATE: a skip runs past its group".into());
            }
        }
        self.local.clear();
        self.outside.clear();
        for i in 1..INDICES as u16 {
            self.flags[usize::from(i)] >>= 1;
            if self.by_index[usize::from(i)].is_some() {
                self.local.push(i);
            } else {
                self.outside.push(i);
            }
        }
        for i in with_masks {
            if let Some(bytes) = read_masks(payload, &mut at)? {
                let appearance = Appearance::decode(&bytes, hidden).map_err(|e| format!("PLAYER_UPDATE: player {i}'s appearance: {e}"))?;
                if let Some(player) = self.by_index[usize::from(i)].as_mut() {
                    player.appearance = Some(appearance);
                }
            }
        }
        if at != payload.len() {
            return Err(format!("PLAYER_UPDATE: read {at} of {} bytes", payload.len()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::bits::BitWriter;
    use crate::protocol::player_update::{appearance_mask, write_local_stay, write_skip};

    fn look() -> Appearance {
        Appearance {
            female: false,
            prefix_title: None,
            postfix_title: None,
            slots: vec![0x100 + 452, 0],
            colours: [0; 10],
            render_emote: 1426,
            name: "bob".into(),
            combat_level: 3,
            summoning_combat_level: 0,
            pvp_threshold: 255,
        }
    }

    /// What a server writes for a client whose only player is itself: its own entry, then the
    /// 2046 others skipped together.
    fn packet(me_updated: bool) -> Vec<u8> {
        let mut w = BitWriter::default();
        if me_updated {
            write_local_stay(&mut w, true);
        } else {
            w.write(1, 0);
            write_skip(&mut w, 0);
        }
        let mut out = w.bytes;
        let mut w = BitWriter::default();
        w.write(1, 0);
        write_skip(&mut w, 2045);
        out.extend(w.bytes);
        if me_updated {
            out.extend(appearance_mask(&look().encode()));
        }
        out
    }

    #[test]
    fn the_own_appearance_arrives_and_later_updates_skip() {
        let hidden = [0, 0];
        let mut players = Players::new(9);
        assert!(players.me().unwrap().appearance.is_none());
        players.update(&packet(true), &hidden).unwrap();
        assert_eq!(players.me().unwrap().appearance, Some(look()));
        // Skipped twice: once from group 1, then from group 2.
        players.update(&packet(false), &hidden).unwrap();
        assert_eq!(players.flags[9], 1);
        players.update(&packet(false), &hidden).unwrap();
        assert_eq!(players.local, vec![9]);
        // A packet with a byte too many is refused, as the reference throws.
        let mut long = packet(false);
        long.push(0);
        assert!(players.update(&long, &hidden).is_err());
    }
}
