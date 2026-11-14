//! The world list this server gives the lobby. There is no reference server, so what is in it
//! is this server's own: every world it runs (`crate::server::worlds`) with the options it was
//! opened with and the players in it now.
//!
//! The list has no field for a port: a world's entry carries its number and a host. Every
//! world here is given the server's host; which port the client then uses for a world is the
//! client's side of the matter.

use crate::protocol::server_prot::{Packet, ServerProt};
use crate::protocol::world_list::{self, Location, WorldEntry, WorldList};
use crate::server::worlds::Hosted;
use std::sync::Arc;

/// The list: every world at `host`, each with its own flags, activity and location.
fn list(host: &str, lobby_port: u16, worlds: &[Arc<Hosted>]) -> WorldList {
    let mut locations: Vec<Location> = Vec::new();
    let mut entries = Vec::new();
    for hosted in worlds {
        let options = &hosted.options;
        // Worlds in the same place share one entry (the list sends each location once).
        let place = Location { country: options.country, name: options.location.clone() };
        let location = locations.iter().position(|l| *l == place).unwrap_or_else(|| {
            locations.push(place);
            locations.len() - 1
        });
        entries.push(WorldEntry {
            number: hosted.number,
            location,
            flags: options.flags(),
            activity: options.activity(),
            host: host.into(),
            // The client logs in to the port it is already using unless the world says another.
            port: (options.port != lobby_port).then_some(options.port),
            players: hosted.world.seats().online() as i32,
        });
    }
    let mut list = WorldList { locations, worlds: entries, checksum: 0 };
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
pub fn reply(held: i32, host: &str, lobby_port: u16, worlds: &[Arc<Hosted>]) -> Packet {
    let list = list(host, lobby_port, worlds);
    (ServerProt::WorldList, world_list::encode(&list, held != list.checksum))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::world_list::{decode, Update, BOTS_ALLOWED, HIDDEN, HIGHLIGHTED, LOOTSHARE, MEMBERS, QUICK_CHAT};
    use crate::server::worlds::{Options, Worlds};

    const LOBBY_PORT: u16 = 5555;

    fn full(host: &str, worlds: &[Arc<Hosted>]) -> WorldList {
        let (_, payload) = reply(0, host, LOBBY_PORT, worlds);
        match decode(&payload[1..], 0) {
            Ok(Update::Full(list)) => list,
            other => panic!("not a full list: {other:?}"),
        }
    }

    #[test]
    fn a_client_with_the_list_gets_only_counts() {
        let worlds = Worlds::new(Default::default(), Default::default(), None);
        let first = worlds.open(Options { port: LOBBY_PORT, ..Options::default() });
        let list = full("127.0.0.1", &worlds.all());
        assert_eq!((list.worlds[0].host.as_str(), list.worlds[0].activity.as_str()), ("127.0.0.1", "-"));
        let _seat = first.world.seats().join();
        let (_, counts) = reply(list.checksum, "127.0.0.1", LOBBY_PORT, &worlds.all());
        assert_eq!(decode(&counts[1..], 1), Ok(Update::Counts(vec![(0, 1)])));
    }

    /// Every option the administrator picked reaches the client's list, and a world opened
    /// later changes the checksum, so a client holding the old list is sent the new one whole.
    #[test]
    fn each_worlds_options_are_in_the_list() {
        let worlds = Worlds::new(Default::default(), Default::default(), None);
        worlds.open(Options { port: LOBBY_PORT, ..Options::default() });
        let before = full("h", &worlds.all());
        worlds.open(Options {
            name: "Trade".into(),
            port: 5556,
            members: false,
            lootshare: true,
            highlighted: true,
            hidden: true,
            quickchat: true,
            bots_allowed: true,
            country: 22,
            location: "United Kingdom".into(),
            ..Options::default()
        });
        let after = full("h", &worlds.all());
        assert_ne!(before.checksum, after.checksum);
        let second = &after.worlds[1];
        assert_eq!((second.number, second.activity.as_str()), (2, "Trade"));
        assert_eq!(second.flags, QUICK_CHAT | BOTS_ALLOWED | LOOTSHARE | HIGHLIGHTED | HIDDEN);
        assert_eq!(after.locations[second.location], Location { country: 22, name: "United Kingdom".into() });
        // The first world is still a members world in its own place.
        assert_eq!(after.worlds[0].flags, MEMBERS);
        assert_eq!(after.locations[after.worlds[0].location].name, "Local");
    }

    /// A world on the lobby's own port is logged in to where the client already is; one on
    /// another port is listed with that port, so the client knows where to go.
    #[test]
    fn a_world_on_another_port_is_listed_with_it() {
        let worlds = Worlds::new(Default::default(), Default::default(), None);
        worlds.open(Options { port: LOBBY_PORT, ..Options::default() });
        worlds.open(Options { port: 5601, ..Options::default() });
        let list = full("127.0.0.1", &worlds.all());
        assert_eq!((list.worlds[0].port, list.worlds[1].port), (None, Some(5601)));
    }

    /// Worlds in the same place are listed once between them.
    #[test]
    fn one_location_is_shared_by_the_worlds_in_it() {
        let worlds = Worlds::new(Default::default(), Default::default(), None);
        for port in [LOBBY_PORT, 5556, 5557] {
            worlds.open(Options { port, ..Options::default() });
        }
        let list = full("h", &worlds.all());
        assert_eq!(list.locations.len(), 1);
        assert!(list.worlds.iter().all(|w| w.location == 0));
    }
}
