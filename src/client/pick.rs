//! Which ground tile is under the mouse, for "Walk here": the part of
//! `GraphNode_Sub1_Sub4_Sub1.iComponentOnGroundTile` that picks the tile (in the deobfuscated
//! client, `MiniMenuBuilder`'s ground-tile entry). The mouse is turned back through the scene's
//! view and projection (`Matrix44.method6527` with the inverse, at the near and the far plane),
//! and the line between the two points is walked until it goes under the ground
//! (`Class4.method288`, `MiniMenu.raycastTerrainIntersection`).
//!
//! The matrices are this client's own (`ground::scene_camera`), the ones the scene is drawn
//! with (`ground::scene_matrix`), so a tile picked is the tile drawn under the mouse. Approximation, as in drawing: the
//! scene fills the whole game area, where the reference uses the scene component's bounds.

use super::camera::View;
use super::ground::{scene_matrix, Ground, SCENE};
use macroquad::prelude::*;

/// `method288`: how far along the line from `near` to `far` (0 at `near`, 1 at `far`) it first
/// goes under the ground, looked for in tenths up to 1.1, then refined in tenths of the tenth
/// it crossed in, `depth` − 1 more times; −1 when it never does. The ground is the player's
/// `plane`, one higher on a bridge tile (`tileMasks[1] & 0x2`). Points are (x, height, y) in
/// fine units of the scene, height negative up.
fn crossing(ground: &Ground, plane: usize, near: [f32; 3], far: [f32; 3], depth: u32) -> f32 {
    let along = [far[0] - near[0], far[1] - near[1], far[2] - near[2]];
    let mut before = [0.0f32; 3];
    let mut t = 0.0f32;
    while t < 1.1 {
        let at = [t * along[0] + near[0], along[1] * t + near[1], near[2] + t * along[2]];
        let (tx, ty) = ((at[0] as i32) >> 9, (at[2] as i32) >> 9);
        if tx > 0 && ty > 0 && tx < SCENE as i32 && ty < SCENE as i32 {
            let bridge = ground.settings()[1][tx as usize * SCENE + ty as usize] & 0x2 != 0;
            let plane = if plane < 3 && bridge { plane + 1 } else { plane };
            if (ground.average_height(plane, at[0] as i32, at[2] as i32) as f32) < at[1] {
                if depth >= 2 {
                    return t - 0.1 + crossing(ground, plane, before, at, depth - 1) * 0.1;
                }
                return t;
            }
        }
        before = at;
        t += 0.1;
    }
    -1.0
}

/// The scene tile under `mouse` (pixels from the game area's top left, `size` its width and
/// height) through `view`, on the player's `plane`; `None` when the line doesn't meet the
/// ground. A player is size 1, so the tile is the point's `>> 9`.
pub fn ground_tile(ground: &Ground, view: &View, size: (i32, i32), plane: usize, mouse: (i32, i32)) -> Option<(i32, i32)> {
    let (w, h) = (size.0.max(1) as f32, size.1.max(1) as f32);
    let inverse = scene_matrix(view, size).inverse();
    // GL's y grows upwards, the mouse's downwards.
    let (nx, ny) = (2.0 * mouse.0 as f32 / w - 1.0, 1.0 - 2.0 * mouse.1 as f32 / h);
    // Back from the drawing's axes (east is −x, up is +y) to the scene's.
    let scene = |p: Vec3| [-p.x, -p.y, p.z];
    let near = scene(inverse.project_point3(vec3(nx, ny, -1.0)));
    let far = scene(inverse.project_point3(vec3(nx, ny, 1.0)));
    let t = crossing(ground, plane, near, far, 4);
    if t <= 0.0 {
        return None;
    }
    let x = ((far[0] - near[0]) * t + near[0]) as i32;
    let y = (t * (far[2] - near[2]) + near[2]) as i32;
    Some((x >> 9, y >> 9))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::camera::Camera;

    /// A tile's centre on the ground, drawn through the view, is picked back as that tile, for
    /// tiles near and far.
    #[test]
    fn a_tile_drawn_under_the_mouse_is_picked() {
        let ground = Ground::flat();
        let focus = (50 * 512 + 256, 40 * 512 + 256);
        let view = Camera::default().view(&ground, focus, 503, 0);
        let matrix = scene_matrix(&view, (765, 503));
        for tile in [(50, 40), (52, 43), (47, 38), (50, 36)] {
            // The tile's centre in the drawing's axes (east is -x), to the screen.
            let ndc = matrix.project_point3(vec3(-((tile.0 * 512 + 256) as f32), 0.0, (tile.1 * 512 + 256) as f32));
            let mouse = (((ndc.x + 1.0) / 2.0 * 765.0) as i32, ((1.0 - ndc.y) / 2.0 * 503.0) as i32);
            assert_eq!(ground_tile(&ground, &view, (765, 503), 0, mouse), Some(tile), "{mouse:?}");
        }
    }
}
