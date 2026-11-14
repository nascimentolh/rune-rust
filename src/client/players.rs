//! The players the client knows (`client.players`, `Class197`'s lists) and `PLAYER_UPDATE`
//! read into them (`PlayerUpdate.decode`).
//!
//! `REGION` makes the client's own player (`Class197.method3203`): the first list
//! (`PLAYER_INDICES`) holds only them, the second (`anIntArray2426`) every other index, and no
//! index was skipped (`aByteArray2424` all 0). Every `PLAYER_UPDATE` then walks the lists in
//! four groups, each from a fresh byte; see `protocol::player_update`.
//!
//! Ported: a player the client has staying where they are or stepping (walk or run), the
//! appearance and movement type masks, and the steps queued on the player
//! (`PlayerEntity.method16129`, `method16132`, `move`). Not yet: moving along the queue frame by
//! frame (`SystemInfo.method13466`), running's in-between steps (`Class512.method8762`, which
//! needs the route finder), teleports, players leaving or coming into view, the other masks.
//! A path that isn't ported stops the packet with an error, where the reference would carry on.

use crate::protocol::bits::BitReader;
use crate::protocol::player_update::{read_local, read_skip, read_step, Appearance, Masks, RUNNING};

/// The highest index + 1 (`client.players` has 2048 entries).
const INDICES: usize = 2048;

/// The scene's size in tiles (`MAP_REGION_DECODER.getSizeX()`, map size 0: 104).
const SCENE: i32 = 104;

/// How many tiles a path holds (`PathingEntity(manager)` calls `this(manager, 10)`).
const PATH: usize = 10;

/// A player entity, as far as it's ported.
#[derive(Default)]
pub struct Player {
    /// Set by `decodeAppearance` (with `displayName` and `model`) once the server sent it.
    pub appearance: Option<Appearance>,
    /// `regionBaseX` / `regionBaseY`, in tiles of the scene: [0] is the tile they stand on or
    /// walk to, then the tiles still to walk before it, newest first.
    pub path: [(i32, i32); PATH],
    /// `anInt10355`: how many of `path`'s tiles are still to be walked.
    pub steps: usize,
    /// `aBool10568` with `anInt10569` / `anInt10570`: a step read with a mask block, taken
    /// after the masks.
    pending: Option<(i32, i32)>,
}

impl Player {
    /// `method16129`: a step to (x, y) at movement type `kind`. Within the scene it goes on the
    /// path (`method16132`; a run's in-between tiles first, `Class512.method8762`); from or to
    /// outside it the player is put there (`move`). The animations it stops and the facing
    /// aren't ported.
    fn walk_to(&mut self, (x, y): (i32, i32), kind: i8) -> Result<(), String> {
        let inside = |(x, y): (i32, i32)| (0..SCENE).contains(&x) && (0..SCENE).contains(&y);
        if !inside((x, y)) || !inside(self.path[0]) {
            self.steps = 0;
            self.path[0] = (x, y);
            return Ok(());
        }
        if kind == RUNNING {
            return Err("PLAYER_UPDATE: the in-between steps of a run (method8762) aren't ported".into());
        }
        // `method16132`: the path moves down one, the new tile goes first.
        if self.steps < PATH - 1 {
            self.steps += 1;
        }
        for i in (1..=self.steps).rev() {
            self.path[i] = self.path[i - 1];
        }
        self.path[0] = (x, y);
        Ok(())
    }
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
    /// `playerMovementTypes`: each player's movement type (`MoveSpeed` id), 0 until a mask sets it.
    movement_types: Vec<i8>,
}

impl Players {
    /// The lists as `REGION` leaves them (`method3203`): the player at `at` (x, y in tiles of
    /// the whole map), less the scene's base (`MapRegion`: (centre chunk - (104 >> 4)) * 8). Its
    /// other players' positions (18 bits each) aren't kept yet: nothing reads them until players
    /// come into view.
    pub fn new(me: u16, at: (i32, i32), centre_chunk: (i32, i32)) -> Self {
        let base = |chunk: i32| (chunk - (SCENE >> 4)) * 8;
        let mut player = Player::default();
        player.path[0] = (at.0 - base(centre_chunk.0), at.1 - base(centre_chunk.1));
        let mut by_index: Vec<Option<Player>> = (0..INDICES).map(|_| None).collect();
        by_index[usize::from(me)] = Some(player);
        Players {
            me,
            by_index,
            local: vec![me],
            outside: (1..INDICES as u16).filter(|&i| i != me).collect(),
            flags: vec![0; INDICES],
            movement_types: vec![0; INDICES],
        }
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
                    let player = self.by_index[usize::from(i)].as_mut().ok_or("PLAYER_UPDATE: an update for nobody")?;
                    match (kind, masks) {
                        (0, true) => {}
                        (0, false) => return Err(format!("PLAYER_UPDATE: player {i} leaving view isn't ported")),
                        (1 | 2, _) => {
                            let (dx, dy) = read_step(&mut r, kind).ok_or("PLAYER_UPDATE runs out")?.offset();
                            let to = (player.path[0].0 + dx, player.path[0].1 + dy);
                            if masks {
                                player.pending = Some(to);
                            } else {
                                player.walk_to(to, self.movement_types[usize::from(i)])?;
                            }
                        }
                        _ => return Err(format!("PLAYER_UPDATE: player {i} teleporting isn't ported")),
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
        // `decodeMasks` then `decodeMasksBody`: the masks in their order, then a step that came
        // with them, at the movement type (the temporary one, mask 0x1000, isn't ported).
        for i in with_masks {
            let masks = Masks::read(payload, &mut at)?;
            let player = self.by_index[usize::from(i)].as_mut().ok_or("PLAYER_UPDATE: masks for nobody")?;
            if let Some(bytes) = masks.appearance {
                let appearance = Appearance::decode(&bytes, hidden).map_err(|e| format!("PLAYER_UPDATE: player {i}'s appearance: {e}"))?;
                player.appearance = Some(appearance);
            }
            if let Some(kind) = masks.movement_type {
                self.movement_types[usize::from(i)] = kind;
            }
            if let Some(to) = player.pending.take() {
                player.walk_to(to, self.movement_types[usize::from(i)])?;
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
    use crate::protocol::player_update::{write_local, write_skip, Step, WALKING};

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

    /// What a server writes for a client whose only player is itself: its own entry (`None`:
    /// skipped), then the 2046 others skipped together, then the mask block.
    fn packet(me: Option<(Step, Option<Masks>)>) -> Vec<u8> {
        let mut w = BitWriter::default();
        match &me {
            Some((step, masks)) => write_local(&mut w, masks.is_some(), *step),
            None => {
                w.write(1, 0);
                write_skip(&mut w, 0);
            }
        }
        let mut out = w.bytes;
        let mut w = BitWriter::default();
        w.write(1, 0);
        write_skip(&mut w, 2045);
        out.extend(w.bytes);
        if let Some((_, Some(masks))) = me {
            out.extend(masks.encode());
        }
        out
    }

    fn arrival() -> Masks {
        Masks { appearance: Some(look().encode()), movement_type: Some(WALKING) }
    }

    #[test]
    fn the_own_appearance_arrives_and_later_updates_skip() {
        let hidden = [0, 0];
        let mut players = Players::new(9, (3222, 3218), (402, 402));
        assert!(players.me().unwrap().appearance.is_none());
        players.update(&packet(Some((Step::Stay, Some(arrival())))), &hidden).unwrap();
        assert_eq!(players.me().unwrap().appearance, Some(look()));
        assert_eq!(players.movement_types[9], WALKING);
        // Skipped twice: once from group 1, then from group 2.
        players.update(&packet(None), &hidden).unwrap();
        assert_eq!(players.flags[9], 1);
        players.update(&packet(None), &hidden).unwrap();
        assert_eq!(players.local, vec![9]);
        // A packet with a byte too many is refused, as the reference throws.
        let mut long = packet(None);
        long.push(0);
        assert!(players.update(&long, &hidden).is_err());
    }

    /// The scene's base is (402 - 6) * 8 = 3168, so 3222, 3218 is 54, 50. A walk north (6)
    /// goes on the path; one with a mask block is taken after the masks.
    #[test]
    fn steps_go_on_the_path() {
        let hidden = [0, 0];
        let mut players = Players::new(9, (3222, 3218), (402, 402));
        assert_eq!(players.me().unwrap().path[0], (54, 50));
        players.update(&packet(Some((Step::Walk(6), None))), &hidden).unwrap();
        players.update(&packet(Some((Step::Walk(7), Some(arrival())))), &hidden).unwrap();
        let me = players.me().unwrap();
        assert_eq!((me.path[0], me.path[1], me.path[2], me.steps), ((55, 52), (54, 51), (54, 50), 2));
        // Running (type 2 at the run movement type) needs the route finder: refused.
        let run = Masks { appearance: None, movement_type: Some(RUNNING) };
        assert!(players.update(&packet(Some((Step::Run(13), Some(run)))), &hidden).is_err());
    }
}
