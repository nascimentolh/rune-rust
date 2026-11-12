//! The world list this server gives the lobby. There is no reference server, so what is in it
//! is this server's own: its one game world, with the players in it now.

use crate::players::World;
use crate::protocol::world_list::{self, Location, WorldEntry, WorldList, MEMBERS};

/// This server's world number. Estimated: a server choice, not from the client.
pub const NUMBER: i32 = 1;
/// The location shown for it. Estimated: country 0 and its name are a server choice.
const COUNTRY: i32 = 0;
const LOCATION: &str = "Local";

/// The list: one members world at `host` (members because every account here is one).
fn list(host: &str, world: &World) -> WorldList {
    let mut list = WorldList {
        locations: vec![Location { country: COUNTRY, name: LOCATION.into() }],
        worlds: vec![WorldEntry {
            number: NUMBER,
            location: 0,
            flags: MEMBERS,
            activity: "-".into(),
            host: host.into(),
            players: world.online() as i32,
        }],
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

/// The `WORLD_LIST` payload for a client holding the list with `held` checksum: player counts
/// only when it already has this list, else the whole list.
pub fn reply(held: i32, host: &str, world: &World) -> Vec<u8> {
    let list = list(host, world);
    world_list::encode(&list, held != list.checksum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::world_list::{decode, Update};

    #[test]
    fn a_client_with_the_list_gets_only_counts() {
        let world = World::new(10);
        let full = reply(0, "127.0.0.1", &world);
        let Ok(Update::Full(list)) = decode(&full[1..], 0) else { panic!("not a full list") };
        assert_eq!(list.worlds[0].host, "127.0.0.1");
        let _seat = world.join();
        let counts = reply(list.checksum, "127.0.0.1", &world);
        assert_eq!(decode(&counts[1..], 1), Ok(Update::Counts(vec![(0, 1)])));
    }
}
