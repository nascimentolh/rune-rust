//! The world list: the lobby asks with `REQUEST_WORLD_LIST` (client 47, 4 bytes: the checksum of
//! the list it already has, 0 at first) and the server answers with `WORLD_LIST` (server 103,
//! u16 length).
//!
//! `WORLD_LIST`, as `PacketDecoder` and `Class151.decodeWorldList` read it: a "last part" byte
//! (1 = the list is whole; 0 = more parts follow and are appended), then the list itself
//! (`MonoStereoPreference.decodeWorldList`):
//! - 2, then 1 when the full list follows (0 = only player counts, the client keeps its list);
//! - the full list (`CutsceneActionType.decodeWorldList`): location count (smart), each
//!   location's country (smart) and name (string); the first and last world number and the
//!   world count (smarts); each world's number − first (smart), location index (u8), flags
//!   (int), activity (string) and host (string); the list's checksum (int);
//! - each world's number − first (smart) and player count (u16, 65535 = offline)
//!   (`Class359.decodePlayerCounts`).
//!
//! Strings are `readGJString`: a 0 byte, the text, a 0 byte. Smarts are one byte below 128,
//! else two bytes − 32768.

/// `Class217` flag bits.
pub const MEMBERS: i32 = 0x1;
pub const LOOTSHARE: i32 = 0x8;

/// A world's location (`World`): its country (which flag shows) and name.
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub country: i32,
    pub name: String,
}

/// One world (`WorldDescriptor`).
#[derive(Clone, Debug, PartialEq)]
pub struct WorldEntry {
    pub number: i32,
    /// Index into the locations.
    pub location: usize,
    pub flags: i32,
    /// "-" when the world has none.
    pub activity: String,
    pub host: String,
    /// -1 = offline.
    pub players: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldList {
    pub locations: Vec<Location>,
    pub worlds: Vec<WorldEntry>,
    pub checksum: i32,
}

/// What a `WORLD_LIST` carries: the whole list, or player counts for the list the client has.
#[derive(Debug, PartialEq)]
pub enum Update {
    Full(WorldList),
    /// (world number, players), numbers as the client's list has them.
    Counts(Vec<(i32, i32)>),
}

fn smart(out: &mut Vec<u8>, v: i32) {
    if (0..128).contains(&v) {
        out.push(v as u8);
    } else {
        out.extend(((v + 32768) as u16).to_be_bytes());
    }
}

fn text(out: &mut Vec<u8>, s: &str) {
    out.push(0);
    out.extend(s.bytes());
    out.push(0);
}

/// The first world number (`WORLD_LIST_START`); 0 when there are none.
fn first(list: &WorldList) -> i32 {
    list.worlds.iter().map(|w| w.number).min().unwrap_or(0)
}

/// The `WORLD_LIST` payload in one part. `full` = false sends only the player counts, for a
/// client whose checksum already matches.
pub fn encode(list: &WorldList, full: bool) -> Vec<u8> {
    let mut out = vec![1, 2, full as u8];
    let start = first(list);
    if full {
        smart(&mut out, list.locations.len() as i32);
        for l in &list.locations {
            smart(&mut out, l.country);
            text(&mut out, &l.name);
        }
        let end = list.worlds.iter().map(|w| w.number).max().unwrap_or(0);
        smart(&mut out, start);
        smart(&mut out, end);
        smart(&mut out, list.worlds.len() as i32);
        for w in &list.worlds {
            smart(&mut out, w.number - start);
            out.push(w.location as u8);
            out.extend(w.flags.to_be_bytes());
            text(&mut out, &w.activity);
            text(&mut out, &w.host);
        }
        out.extend(list.checksum.to_be_bytes());
    }
    for w in &list.worlds {
        smart(&mut out, w.number - start);
        out.extend((if w.players < 0 { 65535 } else { w.players as u16 }).to_be_bytes());
    }
    out
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn u8(&mut self) -> Result<u8, String> {
        let b = *self.data.get(self.at).ok_or("world list ends early")?;
        self.at += 1;
        Ok(b)
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok((self.u8()? as u16) << 8 | self.u8()? as u16)
    }
    fn int(&mut self) -> Result<i32, String> {
        Ok((self.u16()? as i32) << 16 | self.u16()? as i32)
    }
    fn smart(&mut self) -> Result<i32, String> {
        match self.data.get(self.at) {
            Some(&b) if b < 128 => Ok(self.u8()? as i32),
            _ => Ok(self.u16()? as i32 - 32768),
        }
    }
    fn text(&mut self) -> Result<String, String> {
        if self.u8()? != 0 {
            return Err("world list string doesn't start with 0".into());
        }
        let rest = &self.data[self.at..];
        let len = rest.iter().position(|&b| b == 0).ok_or("world list string has no end")?;
        self.at += len + 1;
        Ok(rest[..len].iter().map(|&b| b as char).collect())
    }
}

/// Reads a whole list (the parts joined, without each part's "last part" byte). `known` is the
/// world count of the list the client already has, for a counts-only update.
pub fn decode(data: &[u8], known: usize) -> Result<Update, String> {
    let mut r = Reader { data, at: 0 };
    if r.u8()? != 2 {
        return Err("world list doesn't start with 2".into());
    }
    let full = r.u8()? == 1;
    let mut list = WorldList::default();
    let (mut start, mut count) = (0, known);
    if full {
        for _ in 0..r.smart()? {
            let country = r.smart()?;
            list.locations.push(Location { country, name: r.text()? });
        }
        start = r.smart()?;
        let _end = r.smart()?;
        count = r.smart()? as usize;
        for _ in 0..count {
            let number = start + r.smart()?;
            let location = r.u8()? as usize;
            let flags = r.int()?;
            let activity = r.text()?;
            list.worlds.push(WorldEntry { number, location, flags, activity, host: r.text()?, players: 0 });
        }
        list.checksum = r.int()?;
    }
    let mut counts = Vec::with_capacity(count);
    for _ in 0..count {
        let offset = r.smart()?;
        let players = r.u16()?;
        counts.push((offset, if players == 65535 { -1 } else { players as i32 }));
    }
    if !full {
        return Ok(Update::Counts(counts));
    }
    for (offset, players) in counts {
        if let Some(w) = list.worlds.iter_mut().find(|w| w.number == start + offset) {
            w.players = players;
        }
    }
    Ok(Update::Full(list))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> WorldList {
        WorldList {
            locations: vec![Location { country: 0, name: "Local".into() }],
            worlds: vec![
                WorldEntry { number: 1, location: 0, flags: MEMBERS, activity: "-".into(), host: "127.0.0.1".into(), players: 3 },
                WorldEntry { number: 200, location: 0, flags: 0, activity: "Trade".into(), host: "h".into(), players: -1 },
            ],
            checksum: 77,
        }
    }

    #[test]
    fn a_full_list_round_trips() {
        let p = encode(&sample(), true);
        assert_eq!(p[0], 1);
        assert_eq!(decode(&p[1..], 0), Ok(Update::Full(sample())));
    }

    #[test]
    fn counts_only_keeps_offsets() {
        let p = encode(&sample(), false);
        assert_eq!(decode(&p[1..], 2), Ok(Update::Counts(vec![(0, 3), (199, -1)])));
    }
}
