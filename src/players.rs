//! The players in one game world, server side: who is in it and which player index each has.
//!
//! The client keeps other players in a table of 2048 slots (`Class197`, the 30/18-bit position
//! block of `REGION`), slot 0 unused, so a world holds at most 2047 players. Indices are handed
//! out lowest-free first and given back when the player leaves, so they stay small and are
//! never shared by two players at once.
//!
//! `World::join` returns a `Seat`; dropping it (the connection ended, however it ended) frees
//! the index. The lock is held only to pick or free an index, never during network I/O.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

/// The highest player index the client's tables have room for.
pub const MAX_INDEX: u16 = 2047;

/// One world's player slots.
pub struct World {
    capacity: u16,
    free: Mutex<BTreeSet<u16>>,
}

/// A player's place in a world, held for as long as they are in it.
pub struct Seat {
    world: Arc<World>,
    pub index: u16,
}

impl World {
    /// A world for at most `capacity` players (cut to `MAX_INDEX`).
    pub fn new(capacity: u16) -> Arc<Self> {
        let capacity = capacity.clamp(1, MAX_INDEX);
        Arc::new(World { capacity, free: Mutex::new((1..=capacity).collect()) })
    }

    /// Takes the lowest free index, or `None` when the world is full.
    pub fn join(self: &Arc<Self>) -> Option<Seat> {
        let index = self.free.lock().unwrap().pop_first()?;
        Some(Seat { world: Arc::clone(self), index })
    }

    pub fn online(&self) -> usize {
        self.capacity as usize - self.free.lock().unwrap().len()
    }

    pub fn capacity(&self) -> u16 {
        self.capacity
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        self.world.free.lock().unwrap().insert(self.index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_are_unique_reused_and_capped() {
        let w = World::new(3);
        let a = w.join().unwrap();
        let b = w.join().unwrap();
        let c = w.join().unwrap();
        assert_eq!((a.index, b.index, c.index), (1, 2, 3));
        assert!(w.join().is_none());
        drop(b);
        assert_eq!(w.join().unwrap().index, 2);
        assert_eq!(World::new(u16::MAX).capacity(), MAX_INDEX);
    }

    #[test]
    fn many_players_at_once_get_distinct_indices() {
        let w = World::new(2000);
        let seats: Vec<Seat> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..250).map(|_| s.spawn(|| w.join().unwrap())).collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let distinct: BTreeSet<u16> = seats.iter().map(|s| s.index).collect();
        assert_eq!(distinct.len(), 250);
        assert_eq!(w.online(), 250);
        drop(seats);
        assert_eq!(w.online(), 0);
    }
}
