//! The world list this server gives the lobby. There is no reference server, so what is in it
//! is this server's own: every world it runs (`crate::server::worlds`), each with the players
//! in it now.
//!
//! The list has no field for a port: a world's entry carries its number and a host. Every
//! world here is given the server's host; which port the client then uses for a world is the
//! client's side of the matter.

use crate::protocol::server_prot::{Packet, ServerProt};
use crate::protocol::world_list::{self, Location, WorldEntry, WorldList, LOOTSHARE, MEMBERS};
use crate::server::worlds::Hosted;
use std::sync::Arc;

/// The location shown for every world. Estimated: country 0 and its name are a server choice.
const COUNTRY: i32 = 0;
const LOCATION: &str = "Local";

/// The list: every world at `host`, with its name in the activity column ("-" for none).
fn list(host: &str, worlds: &[Arc<Hosted>]) -> WorldList {
    let entry = |hosted: &Arc<Hosted>| {
        let options = &hosted.options;
        let flags = if options.members { MEMBERS } else { 0 } | if options.lootshare { LOOTSHARE } else { 0 };
        WorldEntry {
            number: hosted.number,
            location: 0,
            flags,
            activity: if options.name.is_empty() { "-".into() } else { options.name.clone() },
            host: host.into(),
            players: hosted.world.seats().online() as i32,
        }
    };
    let mut list = WorldList {
        locations: vec![Location { country: COUNTRY, name: LOCATION.into() }],
        worlds: worlds.iter().map(entry).collect(),
        checksum: 0,
    };
    list.checksum = checksum(&list);
    list
}

/// What the client sends back to say which list it has (`WORLD_LIST_IDK`): any value that
/// changes with the list, never 0 (the client's "none yet").
fn checksum(list: &WorldList) -> i32 {
    let mut fixed = list.clone();
    fixed.worlds.iter_mut().for_each(|w| w.players = 0);
    let h = world_list::encode(&fixed, true).iter().fold(17i32, |h, &b| h.wrapping_mul(31).wrapping_add(b as i32));
    if h == 0 {
        1
    } else {
        h
    }
}

/// `WORLD_LIST` for a client holding the list with `held` checksum: player counts only when it
/// already has this list, else the whole list.
pub fn reply(held: i32, host: &str, worlds: &[Arc<Hosted>]) -> Packet {
    let list = list(host, worlds);
    (ServerProt::WorldList, world_list::encode(&list, held != list.checksum))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::world_list::{decode, Update};
    use crate::server::worlds::{Options, Worlds};

    fn options(name: &str, port: u16, lootshare: bool) -> Options {
        Options { name: name.into(), port, capacity: 10, members: true, lootshare }
    }

    #[test]
    fn a_client_with_the_list_gets_only_counts() {
        let worlds = Worlds::new(Default::default(), Default::default(), None);
        let first = worlds.open(options("", 5555, false));
        let (packet, full) = reply(0, "127.0.0.1", &worlds.all());
        assert_eq!(packet, ServerProt::WorldList);
        let Ok(Update::Full(list)) = decode(&full[1..], 0) else { panic!("not a full list") };
        assert_eq!((list.worlds[0].host.as_str(), list.worlds[0].activity.as_str()), ("127.0.0.1", "-"));
        let _seat = first.world.seats().join();
        let (_, counts) = reply(list.checksum, "127.0.0.1", &worlds.all());
        assert_eq!(decode(&counts[1..], 1), Ok(Update::Counts(vec![(0, 1)])));
    }

    /// A world the administrator opens is listed with its name and flags, and the list's
    /// checksum changes, so a client holding the old list is sent the new one whole.
    #[test]
    fn a_new_world_is_listed_and_changes_the_checksum() {
        let worlds = Worlds::new(Default::default(), Default::default(), None);
        worlds.open(options("", 5555, false));
        let before = list("h", &worlds.all());
        worlds.open(options("Trade", 5556, true));
        let after = list("h", &worlds.all());
        assert_ne!(before.checksum, after.checksum);
        let second = &after.worlds[1];
        assert_eq!((second.number, second.activity.as_str(), second.flags), (2, "Trade", MEMBERS | LOOTSHARE));
        let (_, sent) = reply(before.checksum, "h", &worlds.all());
        assert!(matches!(decode(&sent[1..], 1), Ok(Update::Full(_))));
    }
}
