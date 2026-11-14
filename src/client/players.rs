//! The players the client knows (`client.players`, `Class197`'s lists) and `PLAYER_UPDATE`
//! read into them (`PlayerUpdate.decode`).
//!
//! `REGION` makes the client's own player (`Class197.method3203`): the first list
//! (`PLAYER_INDICES`) holds only them, the second (`anIntArray2426`) every other index, and no
//! index was skipped (`aByteArray2424` all 0). Every `PLAYER_UPDATE` then walks the lists in
//! four groups, each from a fresh byte; see `protocol::player_update`.
//!
//! Ported: a player the client has staying where they are or stepping (walk or run), the
//! appearance and movement type masks, the steps queued on the player
//! (`PlayerEntity.method16129`, `method16132`, `move`), and every 20 ms cycle the player moving
//! along them and turning to face where they go (`Class155.method2636` → `Class363.method6287`:
//! `SystemInfo.method13466`, `MapSpriteIndexLoader.method7188`, `PathingEntity.method15796`).
//! Not yet: running's in-between steps (`Class512.method8762`, which needs the route finder),
//! teleports, players leaving or coming into view, the other masks, and what animations do to
//! the movement (an animation playing holds the steps back; none is played yet). A path that
//! isn't ported stops the packet with an error, where the reference would carry on.

use crate::cache::render_anim::RenderAnim;
use crate::protocol::bits::BitReader;
use crate::protocol::player_update::{read_local, read_skip, read_step, Appearance, Masks, RUNNING};

/// The highest index + 1 (`client.players` has 2048 entries).
const INDICES: usize = 2048;

/// The scene's size in tiles (`MAP_REGION_DECODER.getSizeX()`, map size 0: 104).
const SCENE: i32 = 104;

/// How many tiles a path holds (`PathingEntity(manager)` calls `this(manager, 10)`).
const PATH: usize = 10;

/// `MoveSpeed.HALF_WALK.id`: steps at half speed.
const HALF_WALK: i8 = 0;

/// `PathingEntity.anInt10340`: an entity's own turn speed (256 for every one built).
const TURN_SPEED: i32 = 256;

/// A tile's centre in fine units (512 a tile) for a player, whose size is 1 (`size * 256`).
fn centre((x, y): (i32, i32)) -> (i32, i32) {
    (x * 512 + 256, y * 512 + 256)
}

/// `Class19`: an angle (16384 a turn) that turns towards a target, speeding up and slowing
/// down so it stops there.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Turning {
    /// `turnDirection`.
    pub angle: i32,
    /// `anInt154`: how fast it is turning.
    speed: i32,
}

impl Turning {
    /// `method576`: one step towards `target`, speeding up by `step` (up to `max`, 0 = no
    /// limit) while it can still stop in time, slowing down after. False when it didn't need to
    /// speed up.
    fn towards(&mut self, target: i32, step: i32, max: i32) -> bool {
        let before = self.speed;
        if self.angle == target && self.speed == 0 {
            return false;
        }
        let speeding = if self.speed != 0 {
            let stopping = self.speed.wrapping_mul(self.speed) / (step * 2);
            if self.speed > 0 && target > self.angle {
                let at = stopping + self.angle;
                at < target && at >= self.angle
            } else if self.speed < 0 && target < self.angle {
                let at = self.angle - stopping;
                at > target && at <= self.angle
            } else {
                false
            }
        } else {
            if target > self.angle && target <= step + self.angle || target < self.angle && target >= self.angle - step {
                self.angle = target;
                return false;
            }
            true
        };
        if speeding {
            if target > self.angle {
                self.speed += step;
                if max != 0 && self.speed > max {
                    self.speed = max;
                }
            } else {
                self.speed -= step;
                if max != 0 && self.speed < -max {
                    self.speed = -max;
                }
            }
            if before != self.speed {
                let stopping = self.speed.wrapping_mul(self.speed) / (step * 2);
                if target > self.angle {
                    if stopping + self.angle > target {
                        self.speed = before;
                    }
                } else if target < self.angle && self.angle - stopping < target {
                    self.speed = before;
                }
            }
        } else if self.speed > 0 {
            self.speed = (self.speed - step).max(0);
        } else {
            self.speed = (self.speed + step).min(0);
        }
        self.angle += (before + self.speed) >> 1;
        speeding
    }

    /// `method577`: straight to `angle`, not turning.
    fn set(&mut self, angle: i32) {
        self.angle = angle;
        self.speed = 0;
    }
}

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
    /// `walkTypes`: the movement type each of `path`'s steps is taken at.
    walk_types: [i8; PATH],
    /// Where they are, in fine units of the scene (`method11166().coords` x and z: 512 a tile).
    pub fine: (i32, i32),
    /// `aClass19_10359`: the way they face, and `turnDirection`: the way they are turning to.
    pub facing: Turning,
    turn_direction: i32,
    /// `anInt10368`: the step speed (<< 9) of a base animation set that eases it.
    step_speed: i32,
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
            // `move`: no steps left, put on the tile's centre.
            self.steps = 0;
            self.path[0] = (x, y);
            self.fine = centre((x, y));
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
            self.walk_types[i] = self.walk_types[i - 1];
        }
        self.path[0] = (x, y);
        self.walk_types[0] = kind;
        Ok(())
    }

    /// `method15863`: start turning to face `angle`, the short way round.
    fn face(&mut self, angle: i32, bas: &RenderAnim) {
        if bas.turn_acceleration != 0 || TURN_SPEED != 0 {
            self.facing.angle &= 0x3fff;
            let by = (angle - self.facing.angle) & 0x3fff;
            self.turn_direction = if by > 8192 { self.facing.angle - (16384 - by) } else { self.facing.angle + by };
        }
    }

    /// `turn(direction, false)`: face `direction` at once.
    fn turn(&mut self, direction: i32, bas: &RenderAnim) {
        if bas.turn_acceleration != 0 || TURN_SPEED != 0 {
            self.turn_direction = direction & 0x3fff;
            self.facing.set(self.turn_direction);
        }
    }

    /// `SystemInfo.method13466` for a player (not forced): towards the oldest tile still to
    /// walk, facing it, at 16 a cycle (8 while turning, 24 with more than 2 tiles queued, 32
    /// with more than 3; doubled running, halved at half walk), eased when the base animation
    /// set says so (`anInt2826`); more than two tiles away, it is put there at once.
    fn step_along(&mut self, bas: &RenderAnim) {
        if self.steps == 0 {
            return;
        }
        let (x, z) = self.fine;
        let (tx, tz) = centre(self.path[self.steps - 1]);
        let angle = match (x.cmp(&tx), z.cmp(&tz)) {
            (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => Some(10240),
            (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => Some(14336),
            (std::cmp::Ordering::Less, std::cmp::Ordering::Equal) => Some(12288),
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => Some(6144),
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => Some(2048),
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => Some(4096),
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => Some(8192),
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => Some(0),
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => None,
        };
        if let Some(angle) = angle {
            self.face(angle, bas);
        }
        let kind = self.walk_types[self.steps - 1];
        if (tx - x).abs() > 1024 || (tz - z).abs() > 1024 {
            self.fine = (tx, tz);
            self.turn(self.turn_direction, bas);
            self.steps -= 1;
            return;
        }
        let mut speed = 16;
        // Turning, not facing anyone (`faceEntity` is never set yet), with a turn speed.
        if self.turn_direction - self.facing.angle != 0 && TURN_SPEED != 0 {
            speed = 8;
        }
        if self.steps > 2 {
            speed = 24;
        }
        if self.steps > 3 {
            speed = 32;
        }
        if kind == RUNNING {
            speed <<= 1;
        } else if kind == HALF_WALK {
            speed >>= 1;
        }
        let ease = bas.move_acceleration;
        if ease != -1 {
            speed <<= 9;
            if self.steps == 1 {
                let squared = self.step_speed.wrapping_mul(self.step_speed);
                let far = ((x - tx).abs() << 9).max((z - tz).abs() << 9);
                if squared > far.wrapping_mul(ease).wrapping_mul(2) {
                    self.step_speed /= 2;
                } else if squared / 2 > far {
                    self.step_speed = (self.step_speed - ease).max(0);
                } else if self.step_speed < speed {
                    self.step_speed = (self.step_speed + ease).min(speed);
                }
            } else if self.step_speed < speed {
                self.step_speed = (self.step_speed + ease).min(speed);
            } else if self.step_speed > 0 {
                self.step_speed = (self.step_speed - ease).max(0);
            }
            speed = (self.step_speed >> 9).max(1);
        }
        let toward = |from: i32, to: i32| if from < to { (from + speed).min(to) } else { (from - speed).max(to) };
        self.fine = (toward(x, tx), toward(z, tz));
        if self.fine == (tx, tz) {
            self.steps -= 1;
        }
    }

    /// `method7188` then `method15796`, as far as the facing goes: turn towards
    /// `turnDirection`, speeding up by the set's `anInt2820` up to `anInt2804`, or by the
    /// entity's own turn speed; once it stops moving it settles there. (Facing another entity,
    /// `faceDirection`, and the model's lean, `aClass19_10360` / `aClass19_10330`, aren't ported.)
    fn rotate(&mut self, bas: &RenderAnim) {
        let before = self.facing.angle;
        if bas.turn_acceleration != 0 {
            self.facing.towards(self.turn_direction, bas.turn_acceleration, bas.turn_speed);
        } else {
            self.facing.towards(self.turn_direction, TURN_SPEED, TURN_SPEED);
        }
        if self.facing.angle == before {
            self.facing.set(self.turn_direction);
        }
    }
}

pub struct Players {
    /// `client.myPlayerIndex`.
    me: u16,
    /// `MapRegion.getBase()`: the scene's south-west tile, in tiles of the whole map.
    base: (i32, i32),
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
        let base = ((centre_chunk.0 - (SCENE >> 4)) * 8, (centre_chunk.1 - (SCENE >> 4)) * 8);
        let mut player = Player::default();
        player.path[0] = (at.0 - base.0, at.1 - base.1);
        player.fine = centre(player.path[0]);
        let mut by_index: Vec<Option<Player>> = (0..INDICES).map(|_| None).collect();
        by_index[usize::from(me)] = Some(player);
        Players {
            me,
            base,
            by_index,
            local: vec![me],
            outside: (1..INDICES as u16).filter(|&i| i != me).collect(),
            flags: vec![0; INDICES],
            movement_types: vec![0; INDICES],
        }
    }

    /// `Class155.method2636`, every 20 ms cycle: each player the client has moves
    /// (`Class363.method6287`): along their steps (`method13466`); put back on their tile, with
    /// no steps, when that took them off the scene's edge tile, or for the own player within 12
    /// tiles of the edge; then turning (`method7188`). `bas` gives a base animation set by id;
    /// a player's is their appearance's (`renderEmote`, 0 before it).
    pub fn cycle(&mut self, mut bas: impl FnMut(i32) -> RenderAnim) {
        for &i in &self.local {
            let Some(player) = self.by_index[usize::from(i)].as_mut() else { continue };
            let set = bas(player.appearance.as_ref().map_or(0, |a| i32::from(a.render_emote)));
            player.step_along(&set);
            let (x, z) = player.fine;
            let off = |edge: i32| x < edge * 512 || z < edge * 512 || x >= (SCENE - edge) * 512 || z >= (SCENE - edge) * 512;
            if off(1) || (i == self.me && off(12)) {
                player.fine = centre(player.path[0]);
                player.steps = 0;
            }
            player.rotate(&set);
        }
    }

    /// `MapRegion.getBase()`.
    pub fn base(&self) -> (i32, i32) {
        self.base
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

    /// From standing at a tile's centre, one tile east: it turns to face east (12288 is east
    /// here: x grows) and walks 512 units, 8 a cycle while turning, then 16.
    #[test]
    fn a_step_is_walked_over_cycles_while_turning_to_face_it() {
        let mut p = Player { fine: centre((5, 5)), ..Default::default() };
        p.path[0] = (5, 5);
        p.walk_to((6, 5), 1).unwrap();
        let bas = RenderAnim::default();
        p.step_along(&bas);
        // First cycle: still facing 0, so turning: 8 units.
        assert_eq!(p.fine, (centre((5, 5)).0 + 8, centre((5, 5)).1));
        let mut cycles = 1;
        while p.steps > 0 {
            p.rotate(&bas);
            p.step_along(&bas);
            cycles += 1;
        }
        assert_eq!(p.fine, centre((6, 5)));
        assert!(cycles > 32 && cycles < 64, "{cycles} cycles");
        // `face(12288)` from 0 turns the short way, to -4096; once there, the next `face`
        // keeps the angle within a turn: 12288.
        assert_eq!((p.turn_direction, p.facing.angle), (12288, 12288));
    }

    #[test]
    fn turning_eases_to_its_target() {
        let mut t = Turning::default();
        let mut n = 0;
        while t.angle != 2000 || t.speed != 0 {
            t.towards(2000, 256, 256);
            n += 1;
            assert!(n < 100);
        }
        assert!(n > 4);
    }
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
        // Every cycle it moves along them: two diagonal-ish tiles take a few dozen cycles.
        for _ in 0..200 {
            players.cycle(|_| RenderAnim::default());
        }
        let me = players.me().unwrap();
        assert_eq!((me.fine, me.steps), (centre((55, 52)), 0));
        // Running (type 2 at the run movement type) needs the route finder: refused.
        let run = Masks { appearance: None, movement_type: Some(RUNNING) };
        assert!(players.update(&packet(Some((Step::Run(13), Some(run)))), &hidden).is_err());
    }
}
