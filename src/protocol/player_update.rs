//! `PLAYER_UPDATE` (server packet 43, a u16 length): who each player sees and what changed about
//! them. The client reads it with `PlayerUpdate.decode`: four groups of bit fields, each from a
//! fresh byte (`initBitAccess` … `finishBitAccess`), then a mask block for every player whose
//! update said it has one (`decodeMasks`).
//!
//! The groups walk the client's player lists (`Class197`): the players it has (`PLAYER_INDICES`)
//! whose index wasn't skipped by the last update, then those that were, then the indices it
//! doesn't have (`anIntArray2426`) that were skipped last time, then those that weren't. In each
//! group a player's field starts with one bit: 0 = this player and the next n are skipped (n in
//! `decodeSkip`'s 2 + 5/8/11 bits), 1 = an update follows. Which players were skipped is
//! remembered per index (`aByteArray2424`, bit 1 set while skipped, shifted down after the
//! packet), so both sides keep the same lists; that bookkeeping is the client's and the
//! server's, and this module only holds the fields.
//!
//! Ported so far: a player the client has staying where they are or taking a step
//! (`NewsItem.method1806` types 0, 1 and 2), the appearance mask (0x1) and the movement type
//! mask (0x4). Teleports, players coming into view and the other masks are next.

use super::bits::{BitReader, BitWriter};

/// `decodeSkip`'s field for "this player and the next `n`": 2 bits saying how wide `n` is.
pub fn write_skip(w: &mut BitWriter, n: u32) {
    match n {
        0 => w.write(2, 0),
        1..=31 => {
            w.write(2, 1);
            w.write(5, n);
        }
        32..=255 => {
            w.write(2, 2);
            w.write(8, n);
        }
        _ => {
            w.write(2, 3);
            w.write(11, n);
        }
    }
}

/// `Class399.decodeSkip`.
pub fn read_skip(r: &mut BitReader) -> Option<u32> {
    match r.read(2)? {
        0 => Some(0),
        1 => r.read(5),
        2 => r.read(8),
        _ => r.read(11),
    }
}

/// `method1806`'s walk directions (type 1, 3 bits): the tile offset each one moves by.
const WALK: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

/// Its run directions (type 2, 4 bits): two tiles, or one and two.
const RUN: [(i32, i32); 16] = [
    (-2, -2),
    (-1, -2),
    (0, -2),
    (1, -2),
    (2, -2),
    (-2, -1),
    (2, -1),
    (-2, 0),
    (2, 0),
    (-2, 1),
    (2, 1),
    (-2, 2),
    (-1, 2),
    (0, 2),
    (1, 2),
    (2, 2),
];

/// What a local player did this update (`method1806`'s type).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Type 0 with a mask block: stays where they are.
    Stay,
    /// Type 1: one tile in one of 8 directions (3 bits).
    Walk(u32),
    /// Type 2: two tiles in one go (4 bits).
    Run(u32),
}

impl Step {
    /// The walk direction for a one-tile offset (the reference server's
    /// `getPlayerWalkingDirection`, the same table as the client's).
    pub fn walk(dx: i32, dy: i32) -> Option<Step> {
        WALK.iter().position(|&d| d == (dx, dy)).map(|i| Step::Walk(i as u32))
    }

    /// The tile offset the client moves the player by.
    pub fn offset(self) -> (i32, i32) {
        match self {
            Step::Stay => (0, 0),
            Step::Walk(d) => WALK[d as usize & 7],
            Step::Run(d) => RUN[d as usize & 15],
        }
    }
}

/// `method1806` as the server writes it: the update bit (1), a bit saying a mask block follows,
/// the type in 2 bits, then a walk's 3 or a run's 4 direction bits. A `Stay` needs a mask block:
/// type 0 without one means the player leaves the client's view.
pub fn write_local(w: &mut BitWriter, with_masks: bool, step: Step) {
    w.write(1, 1);
    w.write(1, u32::from(with_masks));
    match step {
        Step::Stay => w.write(2, 0),
        Step::Walk(d) => {
            w.write(2, 1);
            w.write(3, d);
        }
        Step::Run(d) => {
            w.write(2, 2);
            w.write(4, d);
        }
    }
}

/// A local player's update as `method1806` starts reading it: (mask block follows, type).
pub fn read_local(r: &mut BitReader) -> Option<(bool, u32)> {
    Some((r.read(1)? == 1, r.read(2)?))
}

/// The direction after a type 1 (3 bits) or 2 (4 bits).
pub fn read_step(r: &mut BitReader, kind: u32) -> Option<Step> {
    match kind {
        1 => Some(Step::Walk(r.read(3)?)),
        2 => Some(Step::Run(r.read(4)?)),
        _ => None,
    }
}

/// The appearance mask bit (`decodeMasksBody`: `flags & 0x1`).
pub const APPEARANCE: u32 = 0x1;
/// The movement type mask bit (`flags & 0x4`: `playerMovementTypes[index] = read128Byte()`).
pub const MOVEMENT_TYPE: u32 = 0x4;

/// `MoveSpeed.WALKING.id`: the movement type the reference server sends a walking player.
pub const WALKING: i8 = 1;
/// `MoveSpeed.RUNNING.id`.
pub const RUNNING: i8 = 2;

/// The ported masks of one player's block, in `decodeMasksBody`'s order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Masks {
    /// 0x1: the bytes `PlayerEntity.decodeAppearance` reads.
    pub appearance: Option<Vec<u8>>,
    /// 0x4: the movement type (`MoveSpeed` id) the player's steps go at from now on.
    pub movement_type: Option<i8>,
}

impl Masks {
    /// The flags byte (bits 0x8 and 0x100 would say more flag bytes follow; none of the ported
    /// masks needs them), then each mask's data in the client's order: the appearance's length
    /// as `readUnsignedByteC` (sent negated) and its bytes, then the movement type as
    /// `read128Byte` (sent as 128 − the type).
    pub fn encode(&self) -> Vec<u8> {
        let flags = if self.appearance.is_some() { APPEARANCE } else { 0 } | if self.movement_type.is_some() { MOVEMENT_TYPE } else { 0 };
        let mut out = vec![flags as u8];
        if let Some(appearance) = &self.appearance {
            out.push((appearance.len() as u8).wrapping_neg());
            out.extend(appearance);
        }
        if let Some(kind) = self.movement_type {
            out.push(128u8.wrapping_sub(kind as u8));
        }
        out
    }

    /// `decodeMasks`: a block's flags (one byte, a second when 0x8 is set, a third when 0x100
    /// is) and the ported masks. `Err` names a mask that isn't ported.
    pub fn read(data: &[u8], at: &mut usize) -> Result<Masks, String> {
        let byte = |at: &mut usize| -> Result<u32, String> {
            let b = *data.get(*at).ok_or("PLAYER_UPDATE: a mask block runs out")?;
            *at += 1;
            Ok(u32::from(b))
        };
        let mut flags = byte(at)?;
        if flags & 0x8 != 0 {
            flags += byte(at)? << 8;
        }
        if flags & 0x100 != 0 {
            flags += byte(at)? << 16;
        }
        let others = flags & !(APPEARANCE | MOVEMENT_TYPE | 0x8 | 0x100);
        if others != 0 {
            return Err(format!("PLAYER_UPDATE: masks {others:#x} aren't ported"));
        }
        let mut masks = Masks::default();
        if flags & APPEARANCE != 0 {
            let length = byte(at)?.wrapping_neg() as u8 as usize;
            masks.appearance = Some(data.get(*at..*at + length).ok_or("PLAYER_UPDATE: the appearance runs out")?.to_vec());
            *at += length;
        }
        if flags & MOVEMENT_TYPE != 0 {
            masks.movement_type = Some(128u8.wrapping_sub(byte(at)? as u8) as i8);
        }
        Ok(masks)
    }
}

/// A player's appearance as the appearance mask carries it (`PlayerEntity.decodeAppearance`),
/// for a player who isn't wearing an NPC's look.
#[derive(Clone, Debug, PartialEq)]
pub struct Appearance {
    /// Flags bit 0x1. The client keeps it in a field named `male` and hands `male == 1` to the
    /// model; the reference server names the bit `FLAG_FEMALE` (`Rev727AppearanceEncoder`) and
    /// sets it for a player who isn't male.
    pub female: bool,
    /// `prefixTitle` and `postfixTitle` (flags 0x40 and 0x80), around the name in chat
    /// (`getUsernameWithTitle`).
    pub prefix_title: Option<String>,
    pub postfix_title: Option<String>,
    /// One value per equipment slot the cache doesn't hide (`EQUIPMENT_DEFAULTS.hidden[i] != 1`):
    /// 0 = nothing, 0x100 + a body style, 0x4000 + a worn item.
    pub slots: Vec<u16>,
    /// The ten colour choices (`ints_21`).
    pub colours: [u8; 10],
    /// `renderEmote`: the base animation set.
    pub render_emote: u16,
    /// `username` (and `displayName`).
    pub name: String,
    pub combat_level: u8,
    /// `cbLevelWithSumm`, and `cbLevelRelated` (255 = none, read as −1).
    pub summoning_combat_level: u8,
    pub pvp_threshold: u8,
}

impl Appearance {
    /// The bytes `decodeAppearance` reads: flags (size 1: bits 3-5 zero; no skill total), the
    /// titles (`readGJString`: a 0 byte, then the text 0-ended), skull −1, head icon −1, not
    /// hidden, the slots (a 0 byte for nothing, else a u16), no item mesh changes (a u16 0), the
    /// colours, the base animations, the name (Windows-1252, 0-ended), the combat levels, and 0
    /// for "not an NPC".
    pub fn encode(&self) -> Vec<u8> {
        let flags = u8::from(self.female)
            | if self.prefix_title.is_some() { 0x40 } else { 0 }
            | if self.postfix_title.is_some() { 0x80 } else { 0 };
        let mut out = vec![flags];
        for title in [&self.prefix_title, &self.postfix_title].into_iter().flatten() {
            out.push(0);
            out.extend(title.chars().map(crate::cache::text::char_to_cp1252));
            out.push(0);
        }
        out.extend([0xff, 0xff, 0]);
        for &slot in &self.slots {
            if slot == 0 {
                out.push(0);
            } else {
                out.extend(slot.to_be_bytes());
            }
        }
        out.extend(0u16.to_be_bytes());
        out.extend(self.colours);
        out.extend(self.render_emote.to_be_bytes());
        out.extend(self.name.chars().map(crate::cache::text::char_to_cp1252));
        out.push(0);
        out.extend([self.combat_level, self.summoning_combat_level, self.pvp_threshold, 0]);
        out
    }

    /// `decodeAppearance` with the cache's equipment slot table (`EquipmentDefaults.hidden`).
    /// `Err` names what isn't ported (an NPC's look, item mesh changes).
    pub fn decode(data: &[u8], hidden: &[u8]) -> Result<Self, String> {
        let mut at = 0;
        let mut byte = || -> Result<u8, String> {
            let b = *data.get(at).ok_or("appearance runs out")?;
            at += 1;
            Ok(b)
        };
        let flags = byte()?;
        let string = |byte: &mut dyn FnMut() -> Result<u8, String>| -> Result<String, String> {
            let mut text = String::new();
            loop {
                match byte()? {
                    0 => return Ok(text),
                    b => text.push(crate::cache::text::cp1252_to_char(b)),
                }
            }
        };
        // `readGJString`: a 0 byte, then the text.
        let mut titles = [0x40, 0x80].map(|_| None);
        for (title, bit) in titles.iter_mut().zip([0x40, 0x80]) {
            if flags & bit != 0 {
                if byte()? != 0 {
                    return Err("a title doesn't start with a 0 byte".into());
                }
                *title = Some(string(&mut byte)?);
            }
        }
        let [prefix_title, postfix_title] = titles;
        for _ in 0..3 {
            byte()?; // skull, head icon, hidden
        }
        let mut slots = Vec::new();
        for &h in hidden {
            if h == 1 {
                continue;
            }
            let high = byte()?;
            if high == 0 {
                slots.push(0);
                continue;
            }
            let value = u16::from(high) << 8 | u16::from(byte()?);
            if slots.is_empty() && value == 0xffff {
                return Err("an NPC's look isn't ported".into());
            }
            slots.push(value);
        }
        if u16::from(byte()?) << 8 | u16::from(byte()?) != 0 {
            return Err("item mesh changes aren't ported".into());
        }
        let mut colours = [0u8; 10];
        for c in &mut colours {
            *c = byte()?;
        }
        let render_emote = u16::from(byte()?) << 8 | u16::from(byte()?);
        let name = string(&mut byte)?;
        let combat_level = byte()?;
        let (summoning_combat_level, pvp_threshold) = if flags & 0x4 != 0 {
            byte()?;
            byte()?;
            (combat_level, 255)
        } else {
            (byte()?, byte()?)
        };
        Ok(Appearance {
            female: flags & 1 != 0,
            prefix_title,
            postfix_title,
            slots,
            colours,
            render_emote,
            name,
            combat_level,
            summoning_combat_level,
            pvp_threshold,
        })
    }

    /// `getUsernameWithTitle`: the titles around the name.
    pub fn name_with_titles(&self) -> String {
        format!("{}{}{}", self.prefix_title.as_deref().unwrap_or(""), self.name, self.postfix_title.as_deref().unwrap_or(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_and_local_updates_read_back() {
        let mut w = BitWriter::default();
        for n in [0, 5, 200, 2045] {
            write_skip(&mut w, n);
        }
        write_local(&mut w, true, Step::Stay);
        write_local(&mut w, false, Step::Walk(6));
        write_local(&mut w, true, Step::Run(13));
        let mut r = BitReader::new(&w.bytes, 0);
        for n in [0, 5, 200, 2045] {
            assert_eq!(read_skip(&mut r), Some(n));
        }
        assert_eq!(r.read(1), Some(1));
        assert_eq!(read_local(&mut r), Some((true, 0)));
        assert_eq!(r.read(1), Some(1));
        assert_eq!(read_local(&mut r), Some((false, 1)));
        assert_eq!(read_step(&mut r, 1), Some(Step::Walk(6)));
        assert_eq!(r.read(1), Some(1));
        assert_eq!(read_local(&mut r), Some((true, 2)));
        assert_eq!(read_step(&mut r, 2), Some(Step::Run(13)));
    }

    /// The walk table: direction 6 goes north, 1 south; a run's 13 goes two north.
    #[test]
    fn directions_match_their_offsets() {
        assert_eq!(Step::walk(0, 1), Some(Step::Walk(6)));
        assert_eq!(Step::walk(0, -1), Some(Step::Walk(1)));
        assert_eq!(Step::walk(2, 0), None);
        for d in 0..8 {
            let (dx, dy) = Step::Walk(d).offset();
            assert_eq!(Step::walk(dx, dy), Some(Step::Walk(d)));
        }
        assert_eq!(Step::Run(13).offset(), (0, 2));
    }

    #[test]
    fn an_appearance_reads_back_through_its_mask() {
        let mut look = Appearance {
            female: false,
            prefix_title: None,
            postfix_title: None,
            slots: vec![0, 0, 0, 0, 0x100 + 452, 0, 0, 0x100 + 627, 0x100 + 310, 0x100 + 371, 0x100 + 433, 0x100 + 16, 0],
            colours: [12, 218, 218, 180, 110, 0, 0, 0, 0, 0],
            render_emote: 1426,
            name: "bob".into(),
            combat_level: 3,
            summoning_combat_level: 0,
            pvp_threshold: 255,
        };
        let masks = Masks { appearance: Some(look.encode()), movement_type: Some(WALKING) };
        let block = masks.encode();
        let mut at = 0;
        assert_eq!(Masks::read(&block, &mut at), Ok(masks.clone()));
        assert_eq!(at, block.len());
        let bytes = masks.appearance.unwrap();
        // 15 slots in the table, two hidden: 13 sent.
        let hidden = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1];
        assert_eq!(Appearance::decode(&bytes, &hidden), Ok(look.clone()));
        assert!(Masks::read(&[0x40], &mut 0).is_err());
        // The movement type alone: 128 − 1.
        assert_eq!(Masks { appearance: None, movement_type: Some(WALKING) }.encode(), vec![0x4, 127]);
        look.prefix_title = Some("Sir ".into());
        assert_eq!(Appearance::decode(&look.encode(), &hidden).map(|a| a.name_with_titles()), Ok("Sir bob".into()));
    }
}
