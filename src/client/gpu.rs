//! Timing the graphics card for Auto Setup. Port of `Class383.method6513` for toolkit 1 (the
//! reference's OpenGL toolkit; here macroquad's OpenGL): for up to `limit` ms, frames of 136
//! profiling meshes in rows of 1 to 16, each at x = 512·(column − row/2), z = 512·(row + 1);
//! the score is meshes drawn per second.
//!
//! The reference's camera (0, 256, 0), 512 focal length and its light are kept as the
//! projection; faces are drawn white (their colour doesn't change the count). Toolkits 2 (its
//! own native software renderer) and 3 (DirectX) don't exist here, which the reference treats
//! as a toolkit that didn't start (-1).

use crate::cache::mesh::Mesh;
use macroquad::prelude::*;
use std::time::Instant;

/// `Class383.method6513(1, limit)`.
pub fn profile(mesh: &Mesh, limit_ms: u128) -> i32 {
    let (w, h) = (crate::client::interface::SCREEN_WIDTH as u32, crate::client::interface::SCREEN_HEIGHT as u32);
    let target = render_target(w, h);
    // The 512 focal length on the canvas height.
    let fovy = 2.0 * ((h as f32 / 2.0) / 512.0).atan();
    let camera = Camera3D {
        position: vec3(0.0, 256.0, 0.0),
        target: vec3(0.0, 256.0, 1.0),
        up: vec3(0.0, -1.0, 0.0),
        fovy,
        aspect: Some(w as f32 / h as f32),
        render_target: Some(target),
        ..Default::default()
    };
    // One mesh per place in the rows, the translation baked into the vertices.
    let placed: Vec<macroquad::models::Mesh> = (0..=15)
        .rev()
        .flat_map(|row: i32| (0..=row).map(move |col| (col, row)))
        .map(|(col, row)| {
            let (dx, dz) = (512.0 * (col as f32 - row as f32 / 2.0), ((row + 1) * 512) as f32);
            let vertices =
                mesh.vertices.iter().map(|v| Vertex::new(v[0] as f32 + dx, v[1] as f32, v[2] as f32 + dz, 0.0, 0.0, WHITE)).collect();
            let indices = mesh.faces.iter().flatten().copied().collect();
            macroquad::models::Mesh { vertices, indices, texture: None }
        })
        .collect();
    let started = Instant::now();
    let mut drawn: u128 = 0;
    'frames: for _ in 0..500 {
        // Sends the last frame's meshes to the card, then starts a new frame.
        set_camera(&camera);
        clear_background(BLACK);
        for m in &placed {
            draw_mesh(m);
            drawn += 1;
            if started.elapsed().as_millis() >= limit_ms {
                break 'frames;
            }
        }
    }
    set_camera(&camera);
    set_default_camera();
    let elapsed = started.elapsed().as_millis().max(1);
    (drawn * 1000 / elapsed) as i32
}
