//! The game camera, following a focus point (the player). Port of `Isaac.processCamera`
//! (angle limits and the lift over high ground nearby), `LoadingStage.method6683` (where the
//! camera sits for a pitch, yaw and zoom) and `Class492.method8265` (the projection's focal
//! length, `client.anInt7451`). Angles are in steps of 16384 to a turn; distances in the
//! scene's units (512 a tile, height negative up).
//!
//! Not yet: turning the camera (`aFloat7266` / `aFloat7284` come from the game interface's
//! scripts, `Class244.method4195`...), the focus easing towards a moving player
//! (`Class280.method4974`), camera shake, and the bridge plane in the lift
//! (`MapRegion.method4507`, counted as 0).

use super::ground::{Ground, SCENE};
use crate::trig::{cosine, sine};

pub struct Camera {
    /// `client.camAngleX`: pitch.
    pub pitch: f32,
    /// `client.camAngleY`: yaw.
    pub yaw: f32,
    /// `HitbarDefinitions.CAMERA_ZOOM`.
    pub zoom: i32,
    /// `client.anInt7273`: raises the pitch over high ground around the focus.
    lift: i32,
}

impl Default for Camera {
    /// `client.java`: `camAngleX` 1052, `camAngleY` 0; `CAMERA_ZOOM` 600.
    fn default() -> Self {
        Camera { pitch: 1052.0, yaw: 0.0, zoom: 600, lift: 0 }
    }
}

/// Where the camera is and how it projects.
pub struct View {
    /// Camera position (x, height, y).
    pub eye: [i32; 3],
    /// What it looks at: the focus, 200 above the ground there.
    pub focus: [i32; 3],
    /// `anInt7451`: pixels per unit at distance 1.
    pub focal: i32,
}

impl Camera {
    /// `Isaac.processCamera`, once a frame, around fine position `focus` on the player's `plane`.
    pub fn process(&mut self, ground: &Ground, focus: (i32, i32), plane: usize) {
        self.pitch = self.pitch.clamp(1081.0, 2980.0);
        self.yaw = self.yaw.rem_euclid(16384.0);
        let (tx, ty) = (focus.0 >> 9, focus.1 >> 9);
        let here = ground.average_height(plane, focus.0, focus.1);
        let mut most = 0;
        let n = SCENE as i32;
        if tx > 3 && ty > 3 && tx < n - 4 && ty < n - 4 {
            for x in tx - 4..=tx + 4 {
                for y in ty - 4..=ty + 4 {
                    most = most.max(here - ground.corner_height(plane, x as usize, y as usize));
                }
            }
        }
        let target = ((most >> 2) * 1536).clamp(262144, 786432);
        if target > self.lift {
            self.lift += (target - self.lift) / 24;
        } else if target < self.lift {
            self.lift += (target - self.lift) / 80;
        }
    }

    /// `LoadingStage.method6683` with `method8265`'s focal length, for a viewport `height`
    /// pixels tall and fine position `focus` on `plane`.
    pub fn view(&self, ground: &Ground, focus: (i32, i32), height: i32, plane: usize) -> View {
        let pitch = (self.pitch as i32).max(self.lift >> 8);
        let yaw = (self.yaw as i32) & 0x3fff;
        let scale = (height - 334).clamp(0, 100);
        // `aShort7394` 256 and `aShort7324` 320 (`client.java`): the distance grows on tall views.
        let mut distance = ((pitch >> 3) * 3 + self.zoom) << 2;
        distance = (distance * ((320 - 256) * scale / 100 + 256)) >> 8;
        let (a, b) = ((16384 - pitch) & 0x3fff, (16384 - yaw) & 0x3fff);
        let (mut dx, mut dz, mut dy) = (0, 0, distance);
        if a != 0 {
            dz = (sine(a) * -distance) >> 14;
            dy = (cosine(a) * distance) >> 14;
        }
        if b != 0 {
            dx = (sine(b) * dy) >> 14;
            dy = (cosine(b) * dy) >> 14;
        }
        let fz = ground.average_height(plane, focus.0, focus.1) - 200;
        // `aShort7436` 256 and `aShort7437` 205: the focal length shrinks on tall views.
        let focal = height * ((205 - 256) * scale / 100 + 256) / 334;
        View { eye: [focus.0 - dx, fz - dz, focus.1 - dy], focus: [focus.0, fz, focus.1], focal }
    }

    /// `Class209_Sub1.mouseWheelMoved` with Ctrl held: zoom in (up) or out (down) by 30.
    pub fn wheel(&mut self, rotation: i32) {
        if rotation == -1 && self.zoom >= -50 {
            self.zoom -= 30;
        }
        if rotation == 1 && self.zoom < 1700 {
            self.zoom += 30;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_camera_sits_behind_and_above_the_focus() {
        let ground = Ground::flat();
        let mut c = Camera::default();
        c.process(&ground, (52 * 512, 52 * 512), 0);
        assert_eq!(c.pitch, 1081.0);
        let v = c.view(&ground, (52 * 512, 52 * 512), 503, 0);
        // Looking north (yaw 0): south of the focus and above it (height is negative up).
        assert!(v.eye[2] < v.focus[2] && v.eye[1] < v.focus[1] && v.eye[0] == v.focus[0]);
        assert_eq!(v.focal, 503 * 205 / 334);
    }
}
