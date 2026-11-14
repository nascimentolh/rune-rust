//! Route finding for a walker of size 1 (`RouteFinder.findRoute` with `performCalculationS1`):
//! a breadth-first search over a 128 × 128 grid around where they stand, the 8 neighbours of
//! each tile, a step allowed only when the clip flags let them leave and enter
//! (`ClipFlagMap.notFlagged` on the route finder's flags: `BLOCKED_DECO`, `BLOCKED`, `ALT_OBJ`
//! and the walls' `*_ALT_OBJ` sides). The target is one tile (`FixedTileStrategy`). When it
//! can't be reached, the closest tile reached within 10 of it, fewer than 100 steps away, is
//! gone to instead (`findAlternative`). The route is the tiles where it turns, from the first
//! after the start to the end, at most 50 (`client.walkStepsX` holds 50).

use super::collision::{ALT, ALT_OBJ, BLOCKED, BLOCKED_DECO, E, N, NE, NW, S, SE, SW, W};
use std::collections::VecDeque;

const GRID: i32 = 128;
const MAX_TURNS: usize = 50;

/// `directions`' values: the way back to the tile it was reached from.
const BACK_N: u8 = 0x1;
const BACK_E: u8 = 0x2;
const BACK_S: u8 = 0x4;
const BACK_W: u8 = 0x8;

/// What blocks entering a tile from each side: the route finder's flags for those walls.
fn blocks(walls: &[i32]) -> i32 {
    BLOCKED_DECO | BLOCKED | ALT_OBJ | walls.iter().fold(0, |all, &w| all | w << ALT)
}

/// A route from `from` to `to` over `flags(x, y)` (the walker's plane): the tiles where it
/// turns, or `None` when no tile near enough can be reached. `from == to` is an empty route.
pub fn find_route(from: (i32, i32), to: (i32, i32), flags: impl Fn(i32, i32) -> i32) -> Option<Vec<(i32, i32)>> {
    let base = (from.0 - GRID / 2, from.1 - GRID / 2);
    let cell = |x: i32, y: i32| ((x - base.0) * GRID + (y - base.1)) as usize;
    let inside = |x: i32, y: i32| (0..GRID).contains(&(x - base.0)) && (0..GRID).contains(&(y - base.1));
    let mut back = vec![0u8; (GRID * GRID) as usize];
    let mut distance = vec![99_999_999i32; (GRID * GRID) as usize];
    back[cell(from.0, from.1)] = 99;
    distance[cell(from.0, from.1)] = 0;
    let mut queue = VecDeque::from([from]);
    let mut reached = false;
    // `performCalculationS1`: west, east, south, north, then the diagonals, each when the tile
    // (and for a diagonal, both tiles beside it) can be entered from this side.
    let steps: [((i32, i32), u8, &[i32]); 8] = [
        ((-1, 0), BACK_E, &[E]),
        ((1, 0), BACK_W, &[W]),
        ((0, -1), BACK_N, &[N]),
        ((0, 1), BACK_S, &[S]),
        ((-1, -1), BACK_N | BACK_E, &[N, NE, E]),
        ((1, -1), BACK_N | BACK_W, &[NW, N, W]),
        ((-1, 1), BACK_S | BACK_E, &[E, SE, S]),
        ((1, 1), BACK_S | BACK_W, &[S, SW, W]),
    ];
    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == to {
            reached = true;
            break;
        }
        let next = distance[cell(x, y)] + 1;
        for ((dx, dy), way_back, walls) in steps {
            let (nx, ny) = (x + dx, y + dy);
            if !inside(nx, ny) || back[cell(nx, ny)] != 0 || flags(nx, ny) & blocks(walls) != 0 {
                continue;
            }
            // A diagonal also needs the tiles beside it open from this side.
            if dx != 0 && dy != 0 {
                let across = if dx < 0 { E } else { W };
                let along = if dy < 0 { N } else { S };
                if flags(nx, y) & blocks(&[across]) != 0 || flags(x, ny) & blocks(&[along]) != 0 {
                    continue;
                }
            }
            back[cell(nx, ny)] = way_back;
            distance[cell(nx, ny)] = next;
            queue.push_back((nx, ny));
        }
    }
    let mut end = to;
    if !reached {
        // `findAlternative`: the nearest tile reached within 10 of the target (by squared
        // distance, then fewer steps), if any is fewer than 100 steps away.
        let mut best = (i32::MAX, i32::MAX);
        for tx in to.0 - 10..=to.0 + 10 {
            for ty in to.1 - 10..=to.1 + 10 {
                if !inside(tx, ty) || distance[cell(tx, ty)] >= 100 {
                    continue;
                }
                let (ox, oy) = ((tx - to.0).abs(), (ty - to.1).abs());
                let score = (ox * ox + oy * oy, distance[cell(tx, ty)]);
                if score.0 < best.0 || score.0 == best.0 && score.1 < best.1 {
                    best = score;
                    end = (tx, ty);
                }
            }
        }
        if best.0 == i32::MAX {
            return None;
        }
    }
    // Back from the end to the start, keeping the tiles where the way back changes.
    let mut turns = vec![end];
    let (mut x, mut y) = end;
    let mut last = back[cell(x, y)];
    while (x, y) != from {
        let way = back[cell(x, y)];
        if way != last {
            last = way;
            turns.push((x, y));
        }
        if way & BACK_E != 0 {
            x += 1;
        } else if way & BACK_W != 0 {
            x -= 1;
        }
        if way & BACK_N != 0 {
            y += 1;
        } else if way & BACK_S != 0 {
            y -= 1;
        }
    }
    turns.reverse();
    turns.truncate(MAX_TURNS);
    Some(if end == from { Vec::new() } else { turns })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::collision::{Collision, RegionClip};

    /// An open field: straight there, the only turn the end.
    #[test]
    fn an_open_field_is_crossed_in_a_line() {
        assert_eq!(find_route((10, 10), (14, 14), |_, _| 0), Some(vec![(14, 14)]));
        // The search queues the straight steps before the diagonals, so the way back it keeps
        // goes straight first, then diagonally: one turn.
        assert_eq!(find_route((10, 10), (16, 12), |_, _| 0), Some(vec![(14, 10), (16, 12)]));
        assert_eq!(find_route((10, 10), (10, 10), |_, _| 0), Some(Vec::new()));
    }

    /// A wall line x = 12 from y = 0 to 20 with a gap at 15: the route goes through the gap.
    #[test]
    fn a_wall_is_walked_round() {
        let wall = |x: i32, y: i32| if x == 12 && y != 15 && (0..=20).contains(&y) { BLOCKED } else { 0 };
        let route = find_route((10, 10), (14, 10), wall).unwrap();
        assert_eq!(route.last(), Some(&(14, 10)));
        assert!(route.iter().any(|&(x, y)| x == 12 && y == 15) || route.windows(2).any(|w| w[0].1 >= 14 && w[1].1 >= 14));
        // Every tile of a held region blocked but one's own: the nearest reached tile instead.
        let mut c = Collision::default();
        c.insert(RegionClip { id: 0, has_terrain: true, flags: (0..64).map(|y| (0, 30, y, BLOCKED)).collect() });
        let to_wall = find_route((20, 20), (35, 20), |x, y| c.flags(0, x, y)).unwrap();
        assert_eq!(to_wall.last(), Some(&(29, 20)));
        // Nothing reached within 10 of the target: no route.
        assert_eq!(find_route((20, 20), (45, 20), |x, y| c.flags(0, x, y)), None);
    }
}
