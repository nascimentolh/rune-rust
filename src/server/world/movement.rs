//! The movement stage: each player's walk request is routed into steps, and they take this
//! tick's step. Ported from the reference server's `Entity.processEntity` (a walk request is
//! routed, `routeEntityWalkRequest`, and the route's turns become steps, `addSteps`),
//! `addWalkSteps` (from turn to turn, one tile at a time, both axes at once until one is
//! reached) and `processMovement` (one step a tick when walking).
//!
//! The route is found over the map's clip flags (`crate::map`), read by the world's map service
//! (`server::maps`) a region at a time: a request waits a tick or two for the regions it needs,
//! asked for when the player enters and again when they walk. The reference server routes with
//! its `PathFinder`; this server uses the client's own route finder (`RouteFinder.findRoute`:
//! the same 128-tile search with the nearest reachable tile as a fallback), its own choice.
//!
//! Also its own choices, for now:
//! - Running: one step a tick, always (the run orb and the run key aren't ported).
//! - A new map around the player: the reference sends a new `REGION` when the player comes
//!   within two chunks of the scene's edge (`needMapUpdate`); this server doesn't yet, so the
//!   steps stop before that band.
//! - With no map service (the tests, or a server without a cache), nothing blocks.

use super::player::Player;
use crate::map::collision::{region_of, Collision};
use crate::map::route::find_route;
use crate::protocol::player_update::Step;
use crate::server::maps::MapService;
use std::collections::HashSet;

/// The scene's size in chunks (104 tiles), and the chunks at each edge that make the reference
/// load a new map (`needMapUpdate`: `offset < 2 || offset >= size / 8 - 2`).
const SCENE_CHUNKS: i32 = 104 / 8;
const EDGE_CHUNKS: i32 = 2;

/// How far a route may search from the walker (`RouteFinder`'s 128-tile grid, centred).
const SEARCH: i32 = 64;

/// The world's map: the clip flags of the regions read so far, and its service.
pub struct WorldMap {
    collision: Collision,
    service: Option<MapService>,
    asked: HashSet<u32>,
}

impl WorldMap {
    pub fn new(service: Option<MapService>) -> Self {
        WorldMap { collision: Collision::default(), service, asked: HashSet::new() }
    }

    /// The regions a route from `tile` may search.
    fn around((x, y): (i32, i32)) -> impl Iterator<Item = u32> {
        ((x - SEARCH) >> 6..=(x + SEARCH - 1) >> 6)
            .flat_map(move |rx| ((y - SEARCH) >> 6..=(y + SEARCH - 1) >> 6).map(move |ry| region_of(rx << 6, ry << 6)))
    }

    /// Asks for the regions around `tile` not asked for yet, and takes in those read since.
    /// True when every one is held (always, with no service).
    pub fn prepare(&mut self, tile: (i32, i32)) -> bool {
        let Some(service) = &self.service else { return true };
        while let Some(clip) = service.take() {
            self.collision.insert(clip);
        }
        let mut ready = true;
        for region in Self::around(tile) {
            if !self.collision.holds(region) {
                ready = false;
                if self.asked.insert(region) {
                    service.request(region);
                }
            }
        }
        ready
    }

    /// The clip flags of a tile (none with no service).
    fn flags(&self, plane: usize, x: i32, y: i32) -> i32 {
        if self.service.is_none() {
            0
        } else {
            self.collision.flags(plane, x, y)
        }
    }
}

/// Whether a tile is inside the scene the player's client has, away from the edge band.
fn in_scene(scene_chunk: (i32, i32), x: i32, y: i32) -> bool {
    let inside = |tile: i32, centre: i32| {
        let offset = (tile >> 3) - (centre - SCENE_CHUNKS / 2);
        (EDGE_CHUNKS..SCENE_CHUNKS - EDGE_CHUNKS).contains(&offset)
    };
    inside(x, scene_chunk.0) && inside(y, scene_chunk.1)
}

/// `addWalkSteps`: from the last tile queued (or where they stand) towards `to`, x and y each
/// moving one tile closer per step, until it is reached; false when a step would leave the
/// scene's inner part (then `addSteps` stops).
fn add_walk_steps(player: &mut Player, (to_x, to_y): (i32, i32)) -> bool {
    let (mut x, mut y) = player.walk_steps.back().copied().unwrap_or((player.tile.0, player.tile.1));
    while (x, y) != (to_x, to_y) {
        x += (to_x - x).signum();
        y += (to_y - y).signum();
        if !in_scene(player.scene_chunk, x, y) {
            return false;
        }
        player.walk_steps.push_back((x, y));
    }
    true
}

/// Stage: every player, in index order. A walk request whose regions are held is routed and
/// replaces the steps left (`resetWalkSteps`, then `addSteps` over the route's turns); one not
/// held yet waits; then one step is taken (`processMovement`).
pub fn move_players(players: &mut [&mut Player], map: &mut WorldMap) {
    for player in players.iter_mut() {
        player.step = Step::Stay;
        let (x, y, plane) = player.tile;
        let ready = map.prepare((x, y));
        if let Some(destination) = player.walk_request.filter(|_| ready) {
            player.walk_request = None;
            player.walk_steps.clear();
            let plane = plane.clamp(0, 3) as usize;
            for turn in find_route((x, y), destination, |tx, ty| map.flags(plane, tx, ty)).unwrap_or_default() {
                if !add_walk_steps(player, turn) {
                    break;
                }
            }
        }
        let Some((nx, ny)) = player.walk_steps.pop_front() else { continue };
        match Step::walk(nx - x, ny - y) {
            Some(step) => {
                player.tile = (nx, ny, plane);
                player.step = step;
            }
            None => player.walk_steps.clear(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_walk_follows_its_route_one_tile_a_tick() {
        let mut player = Player::new(1, "bob".into(), Some(1));
        let (x, y, _) = player.tile;
        player.walk_request = Some((x + 2, y + 1));
        let mut taken = Vec::new();
        for _ in 0..3 {
            move_players(&mut [&mut player], &mut WorldMap::new(None));
            taken.push(player.step);
        }
        // The route goes straight first (east, 4), then north-east (7), then standing.
        assert_eq!(taken, vec![Step::Walk(4), Step::Walk(7), Step::Stay]);
        assert_eq!(player.tile, (x + 2, y + 1, 0));
    }

    #[test]
    fn steps_stop_before_the_scene_edge() {
        let mut player = Player::new(1, "bob".into(), Some(1));
        let (x, y, _) = player.tile;
        player.walk_request = Some((x + 40, y));
        move_players(&mut [&mut player], &mut WorldMap::new(None));
        let last = player.walk_steps.back().copied().unwrap_or((player.tile.0, player.tile.1));
        // The scene's base is 6 chunks west of its centre chunk; steps end in its 11th chunk.
        let base_x = ((x >> 3) - 6) * 8;
        assert_eq!(last.0, base_x + 11 * 8 - 1);
        // Beyond the route's 128-tile search, with nothing near it reached: no steps at all.
        let mut far = Player::new(2, "ann".into(), Some(1));
        far.walk_request = Some((x + 100, y));
        move_players(&mut [&mut far], &mut WorldMap::new(None));
        assert!(far.walk_steps.is_empty() && far.step == Step::Stay);
    }
}
