//! The movement stage: each player's walk request is turned into steps, and they take this
//! tick's step. Ported from the reference server's `Entity.processEntity` (a walk request
//! becomes steps, `addSteps`), `addWalkSteps` (one tile at a time, both axes at once until one is
//! reached) and `processMovement` (one step a tick when walking).
//!
//! What this server leaves out, for now, and does instead (its own choices):
//! - Collision: the reference routes with `PathFinder` over the map's collision flags; this
//!   server has no collision map yet, so the steps go straight at the destination, through
//!   walls.
//! - Running: one step a tick, always (the run orb and the run key aren't ported).
//! - A new map around the player: the reference sends a new `REGION` when the player comes
//!   within two chunks of the scene's edge (`needMapUpdate`); this server doesn't yet, so the
//!   steps stop before that band.

use super::player::Player;
use crate::protocol::player_update::Step;

/// The scene's size in chunks (104 tiles), and the chunks at each edge that make the reference
/// load a new map (`needMapUpdate`: `offset < 2 || offset >= size / 8 - 2`).
const SCENE_CHUNKS: i32 = 104 / 8;
const EDGE_CHUNKS: i32 = 2;

/// Whether a tile is inside the scene the player's client has, away from the edge band.
fn in_scene(scene_chunk: (i32, i32), x: i32, y: i32) -> bool {
    let inside = |tile: i32, centre: i32| {
        let offset = (tile >> 3) - (centre - SCENE_CHUNKS / 2);
        (EDGE_CHUNKS..SCENE_CHUNKS - EDGE_CHUNKS).contains(&offset)
    };
    inside(x, scene_chunk.0) && inside(y, scene_chunk.1)
}

/// `addWalkSteps`: from the last tile queued (or where they stand) towards the destination,
/// x and y each moving one tile closer per step, until it is reached.
fn add_walk_steps(player: &mut Player, (to_x, to_y): (i32, i32)) {
    let (mut x, mut y) = player.walk_steps.back().copied().unwrap_or((player.tile.0, player.tile.1));
    while (x, y) != (to_x, to_y) {
        x += (to_x - x).signum();
        y += (to_y - y).signum();
        if !in_scene(player.scene_chunk, x, y) {
            return;
        }
        player.walk_steps.push_back((x, y));
    }
}

/// Stage: every player, in index order. A new walk request replaces the steps left
/// (`resetWalkSteps`, then `addSteps`); then one step is taken (`processMovement`).
pub fn move_players(players: &mut [&mut Player]) {
    for player in players.iter_mut() {
        player.step = Step::Stay;
        if let Some(destination) = player.walk_request.take() {
            player.walk_steps.clear();
            add_walk_steps(player, destination);
        }
        let Some((x, y)) = player.walk_steps.pop_front() else { continue };
        let (from_x, from_y, plane) = player.tile;
        match Step::walk(x - from_x, y - from_y) {
            Some(step) => {
                player.tile = (x, y, plane);
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
    fn a_walk_goes_diagonally_then_straight_one_tile_a_tick() {
        let mut player = Player::new(1, "bob".into(), Some(1));
        let (x, y, _) = player.tile;
        player.walk_request = Some((x + 2, y + 1));
        let mut taken = Vec::new();
        for _ in 0..3 {
            move_players(&mut [&mut player]);
            taken.push(player.step);
        }
        // North-east (7), then east (4), then standing.
        assert_eq!(taken, vec![Step::Walk(7), Step::Walk(4), Step::Stay]);
        assert_eq!(player.tile, (x + 2, y + 1, 0));
    }

    #[test]
    fn steps_stop_before_the_scene_edge() {
        let mut player = Player::new(1, "bob".into(), Some(1));
        let (x, y, _) = player.tile;
        player.walk_request = Some((x + 100, y));
        move_players(&mut [&mut player]);
        let last = player.walk_steps.back().copied().unwrap_or((player.tile.0, player.tile.1));
        // The scene's base is 6 chunks west of its centre chunk; steps end in its 11th chunk.
        let base_x = ((x >> 3) - 6) * 8;
        assert_eq!(last.0, base_x + 11 * 8 - 1);
    }
}
