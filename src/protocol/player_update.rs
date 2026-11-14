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
//! Ported so far: a player the client has, staying where they are, with a mask block
//! (`NewsItem.method1806` type 0), and the appearance mask (0x1). Movement, players coming into
//! view and the other masks are next.

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

/// `NewsItem.method1806`, the part that is ported: after the update bit (1), a bit saying a mask
/// block follows, then the movement kind in 2 bits (0 = stays where they are).
pub fn write_local_stay(w: &mut BitWriter, with_masks: bool) {
    w.write(1, 1);
    w.write(1, u32::from(with_masks));
    w.write(2, 0);
}

/// A local player's update as `method1806` starts reading it: (mask block follows, kind).
pub fn read_local(r: &mut BitReader) -> Option<(bool, u32)> {
    Some((r.read(1)? == 1, r.read(2)?))
}

/// The appearance mask bit (`decodeMasksBody`: `flags & 0x1`).
pub const APPEARANCE: u32 = 0x1;

/// A mask block with only the appearance: the flags byte (0x1; bits 0x8 and 0x100 would say more
/// flag bytes follow), then the appearance's length as `readUnsignedByteC` (sent negated) and its
/// bytes (`PlayerEntity.decodeAppearance` reads them).
pub fn appearance_mask(appearance: &[u8]) -> Vec<u8> {
    let mut out = vec![APPEARANCE as u8, (appearance.len() as u8).wrapping_neg()];
    out.extend(appearance);
    out
}

/// `decodeMasks`: a mask block's flags (one byte, a second when 0x8 is set, a third when 0x100
/// is) and, from the ported masks, the appearance's bytes. `Err` names a mask that isn't ported.
pub fn read_masks(data: &[u8], at: &mut usize) -> Result<Option<Vec<u8>>, String> {
    let mut byte = || -> Result<u32, String> {
        let b = *data.get(*at).ok_or("PLAYER_UPDATE: a mask block runs out")?;
        *at += 1;
        Ok(u32::from(b))
    };
    let mut flags = byte()?;
    if flags & 0x8 != 0 {
        flags += byte()? << 8;
    }
    if flags & 0x100 != 0 {
        flags += byte()? << 16;
    }
    let others = flags & !(APPEARANCE | 0x8 | 0x100);
    if others != 0 {
        return Err(format!("PLAYER_UPDATE: masks {others:#x} aren't ported"));
    }
    if flags & APPEARANCE == 0 {
        return Ok(None);
    }
    let length = byte()?.wrapping_neg() as u8 as usize;
    let bytes = data.get(*at..*at + length).ok_or("PLAYER_UPDATE: the appearance runs out")?.to_vec();
    *at += length;
    Ok(Some(bytes))
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
        write_local_stay(&mut w, true);
        let mut r = BitReader::new(&w.bytes, 0);
        for n in [0, 5, 200, 2045] {
            assert_eq!(read_skip(&mut r), Some(n));
        }
        assert_eq!(r.read(1), Some(1));
        assert_eq!(read_local(&mut r), Some((true, 0)));
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
        let block = appearance_mask(&look.encode());
        let mut at = 0;
        let bytes = read_masks(&block, &mut at).unwrap().unwrap();
        assert_eq!(at, block.len());
        // 15 slots in the table, two hidden: 13 sent.
        let hidden = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1];
        assert_eq!(Appearance::decode(&bytes, &hidden), Ok(look.clone()));
        assert!(read_masks(&[0x40], &mut 0).is_err());
        look.prefix_title = Some("Sir ".into());
        assert_eq!(Appearance::decode(&look.encode(), &hidden).map(|a| a.name_with_titles()), Ok("Sir bob".into()));
    }
}
