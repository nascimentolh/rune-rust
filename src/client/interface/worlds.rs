//! The world list as the lobby's scripts see it (`ConnectionInfo.WORLD_DESCRIPTORS_BYID` and
//! friends): asked for once a second at most, received in `WORLD_LIST`, sorted by the World
//! Select columns and walked one world at a time.

use crate::client::text_order::compare;
use crate::protocol::client_packets::request_world_list;
use crate::protocol::client_prot::Packet;
use crate::protocol::world_list::{self, Update, WorldEntry, WorldList, LOOTSHARE, MEMBERS};
use std::time::{Duration, Instant};

/// `method15437`: a list younger than this isn't asked for again.
const FRESH: Duration = Duration::from_millis(1000);

#[derive(Default)]
pub struct Worlds {
    /// `Class244.aBool3007` + the descriptors: `None` until a whole list has arrived.
    list: Option<WorldList>,
    /// `WORLD_DESCRIPTORS_BYID`: indices into the list's worlds, in shown order.
    order: Vec<usize>,
    /// `CURRENT_WORLD`: the next entry of `order` the "next world" op gives.
    cursor: usize,
    /// `aBool5422`: asked, no answer yet.
    waiting: bool,
    /// `aLong5425`: when the last whole list arrived.
    received: Option<Instant>,
    /// `WORLD_LIST_BUFFER`: parts received so far.
    parts: Vec<u8>,
}

impl Worlds {
    /// `method15437` in the lobby: 1 = the list is fresh, 0 = asked (or still waiting). A
    /// request is the packet to send: the checksum of the list held (`WORLD_LIST_IDK`).
    pub fn request(&mut self, now: Instant) -> (i32, Option<Packet>) {
        if self.waiting {
            return (0, None);
        }
        if self.received.is_some_and(|t| now.duration_since(t) < FRESH) {
            return (1, None);
        }
        self.waiting = true;
        let checksum = self.list.as_ref().map_or(0, |l| l.checksum);
        (0, Some(request_world_list(checksum)))
    }

    /// `WORLD_LIST` (`Class151.decodeWorldList`): parts are kept until the last one.
    pub fn receive(&mut self, payload: &[u8], now: Instant) -> Result<(), String> {
        let (&last, data) = payload.split_first().ok_or("empty world list packet")?;
        self.parts.extend_from_slice(data);
        if last != 1 {
            return Ok(());
        }
        let data = std::mem::take(&mut self.parts);
        let known = self.list.as_ref().map_or(0, |l| l.worlds.len());
        match world_list::decode(&data, known)? {
            Update::Full(list) => self.list = Some(list),
            Update::Counts(counts) => {
                if let Some(list) = &mut self.list {
                    let start = list.worlds.iter().map(|w| w.number).min().unwrap_or(0);
                    for (offset, players) in counts {
                        if let Some(w) = list.worlds.iter_mut().find(|w| w.number == start + offset) {
                            w.players = players;
                        }
                    }
                }
            }
        }
        // By world number, as the client fills the array from `WORLD_LIST_START` up.
        let worlds = self.list.as_ref().map_or(&[][..], |l| &l.worlds[..]);
        self.order = (0..worlds.len()).collect();
        self.order.sort_by_key(|&i| worlds[i].number);
        self.cursor = 0;
        self.waiting = false;
        self.received = Some(now);
        Ok(())
    }

    /// `LocationIndexLoader.getWorld`.
    pub fn world(&self, number: i32) -> Option<View<'_>> {
        let list = self.list.as_ref()?;
        list.worlds.iter().find(|w| w.number == number).map(|w| View { list, world: w })
    }

    /// `SceneryShadowPreference.method12629`: back to the first world, and it.
    pub fn first(&mut self) -> Option<View<'_>> {
        self.cursor = 0;
        self.next()
    }

    /// `getCurrentWorldDescriptor`: the next world in shown order.
    pub fn next(&mut self) -> Option<View<'_>> {
        let &i = self.order.get(self.cursor)?;
        self.cursor += 1;
        let list = self.list.as_ref()?;
        Some(View { list, world: &list.worlds[i] })
    }

    /// `Class365.method6299`: sort by a column, then by a second one (-1 = none).
    pub fn sort(&mut self, column: i32, descending: bool, then: i32, then_descending: bool) {
        let Some(list) = &self.list else { return };
        let cmp = |a: usize, b: usize| {
            let (a, b) = (View { list, world: &list.worlds[a] }, View { list, world: &list.worlds[b] });
            let first = column_order(&a, &b, column, descending);
            if first != 0 {
                return if descending { -first } else { first };
            }
            if then == -1 {
                return 0;
            }
            let second = column_order(&a, &b, then, then_descending);
            if then_descending {
                -second
            } else {
                second
            }
        };
        quicksort(&mut self.order, cmp);
    }
}

/// A world with its location, for reading.
pub struct View<'a> {
    list: &'a WorldList,
    world: &'a WorldEntry,
}

impl View<'_> {
    pub fn number(&self) -> i32 {
        self.world.number
    }
    pub fn flags(&self) -> i32 {
        self.world.flags
    }
    pub fn activity(&self) -> &str {
        &self.world.activity
    }
    pub fn country(&self) -> i32 {
        self.list.locations.get(self.world.location).map_or(0, |l| l.country)
    }
    pub fn location(&self) -> &str {
        self.list.locations.get(self.world.location).map_or("", |l| &l.name)
    }
    pub fn players(&self) -> i32 {
        self.world.players
    }
    /// `WorldDescriptor.ping`: -1 until measured.
    ///
    /// ponytail: worlds aren't pinged yet (`ConnectionInfo` ping requests), so always -1.
    pub fn ping(&self) -> i32 {
        -1
    }
    pub fn host(&self) -> &str {
        &self.world.host
    }

    /// The port this world is logged in to, when the list gave it one of its own.
    pub fn port(&self) -> Option<u16> {
        self.world.port
    }

    /// Where a world login for this world goes: its host, and its own port when it has one,
    /// else `port` (the one this client is already using).
    pub fn address(&self, port: u16) -> String {
        format!("{}:{}", self.host(), self.port().unwrap_or(port))
    }
}

/// `Preference_Sub1.method12620`: one column's order.
fn column_order(a: &View, b: &View, column: i32, descending: bool) -> i32 {
    let flag = |bit: i32| match (a.flags() & bit != 0, b.flags() & bit != 0) {
        (true, false) => 1,
        (false, true) => -1,
        _ => 0,
    };
    match column {
        1 => {
            let count = |v: i32| if !descending && v == -1 { 2001 } else { v };
            count(a.players()) - count(b.players())
        }
        2 => compare(a.location(), b.location()),
        3 => match (a.activity() == "-", b.activity() == "-") {
            (true, true) => 0,
            (true, false) => {
                if descending {
                    -1
                } else {
                    1
                }
            }
            (false, true) => {
                if descending {
                    1
                } else {
                    -1
                }
            }
            _ => compare(a.activity(), b.activity()),
        },
        4 => flag(LOOTSHARE),
        5 => flag(0x2),
        6 => flag(0x4),
        7 => flag(MEMBERS),
        8 => {
            let ping = |v: i32| match (descending, v) {
                (true, 1000) => -1,
                (false, -1) => 1000,
                _ => v,
            };
            ping(a.ping()) - ping(b.ping())
        }
        _ => a.number() - b.number(),
    }
}

/// `Class52.method1086`: the client's quicksort (middle pivot moved to the end; ties go left),
/// kept as is so worlds that compare equal end up in the same order as in the original.
fn quicksort(v: &mut [usize], cmp: impl Fn(usize, usize) -> i32 + Copy) {
    if v.len() < 2 {
        return;
    }
    let hi = v.len() - 1;
    v.swap((hi) / 2, hi);
    let pivot = v[hi];
    let mut store = 0;
    for i in 0..hi {
        if cmp(v[i], pivot) <= 0 {
            v.swap(i, store);
            store += 1;
        }
    }
    v.swap(hi, store);
    let (left, right) = v.split_at_mut(store);
    quicksort(left, cmp);
    quicksort(&mut right[1..], cmp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::world_list::{encode, Location};

    fn worlds() -> Worlds {
        let list = WorldList {
            locations: vec![Location { country: 1, name: "Here".into() }],
            worlds: [(3, 10), (1, 50), (2, -1)]
                .map(|(number, players)| WorldEntry {
                    number,
                    location: 0,
                    flags: 0,
                    activity: "-".into(),
                    host: "h".into(),
                    port: (number == 2).then_some(5602),
                    players,
                })
                .to_vec(),
            checksum: 9,
        };
        let mut w = Worlds::default();
        let now = Instant::now();
        assert_eq!(w.request(now), (0, Some(request_world_list(0))));
        assert_eq!(w.request(now), (0, None));
        w.receive(&encode(&list, true), now).unwrap();
        assert_eq!(w.request(now).0, 1);
        w
    }

    fn numbers(w: &mut Worlds) -> Vec<i32> {
        let mut out = vec![w.first().unwrap().number()];
        while let Some(v) = w.next() {
            out.push(v.number());
        }
        out
    }

    #[test]
    fn worlds_start_in_number_order_and_sort_by_players() {
        let mut w = worlds();
        assert_eq!(numbers(&mut w), [1, 2, 3]);
        // Ascending players: offline (-1) counts as 2001, so it goes last.
        w.sort(1, false, -1, false);
        assert_eq!(numbers(&mut w), [3, 1, 2]);
        w.sort(1, true, -1, false);
        assert_eq!(numbers(&mut w), [1, 3, 2]);
        assert_eq!(w.world(2).unwrap().players(), -1);
        assert_eq!(w.world(1).unwrap().country(), 1);
    }
}
