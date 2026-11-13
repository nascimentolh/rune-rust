//! The scene built on entering the world (`MapRegion`'s scene load): the ground, the
//! locations, the drawn pieces and the tile slots, and the roof map kept up to date each frame.

use super::camera::View;
use super::ground::{self, Ground, SCENE};
use super::interface::assets::Assets;
use super::locations::{self, Placed};
use super::login::Arrival;
use super::preferences::Preferences;
use super::scene::{Roofs, Scene, Slots};

pub struct World {
    pub ground: Ground,
    pub locations: Vec<Placed>,
    pub scene: Scene,
    slots: Slots,
    /// `removeRoofsOptionOverride`, set from the remove-roofs preference when the game starts
    /// (`Class190`).
    roof_mode: i32,
}

impl World {
    /// Loads the scene around `arrival` with the player's `prefs`; `brightness` is the colour
    /// table's.
    pub fn build(assets: &mut Assets, prefs: &Preferences, arrival: &Arrival, brightness: f64) -> World {
        let light = ground::Light::scene(prefs.brightness);
        let mut ground = Ground::load(assets, arrival.chunk, brightness, &light, prefs.ground_blending == 1);
        let locations = locations::load(assets, arrival.chunk, &arrival.keys);
        let detail = locations::Detail { ground_decor: prefs.ground_decor != 0, textures: prefs.textures != 0 };
        let (scenery, slots) = locations::scenery(&locations, assets, &ground, &light, brightness, &detail);
        let mut pieces = std::mem::take(&mut ground.pieces);
        pieces.extend(scenery);
        World { ground, locations, scene: Scene::new(pieces), slots, roof_mode: prefs.remove_roofs }
    }

    /// `QuickchatMessageLoader.method7215` and the draw's hiding: the roof map for the player at
    /// fine position `player` on `plane` and the camera's `view`, then what is shown.
    pub fn hide_roofs(&mut self, view: &View, player: (i32, i32), plane: usize) {
        // `CAM_MOVE_ABSOLUTEX` / `Y` are kept inside the scene (`HitbarDefinitions`).
        let edge = (SCENE as i32) * 512 - 1;
        let camera = (view.eye[0].clamp(0, edge) >> 9, view.eye[2].clamp(0, edge) >> 9);
        let tile = |v: i32| ((v >> 9).clamp(0, SCENE as i32 - 1)) as usize;
        let roofs = Roofs::compute(
            self.roof_mode,
            self.ground.settings(),
            &self.slots,
            plane,
            (tile(player.0), tile(player.1)),
            camera,
            view.pitch,
        );
        self.scene.show(roofs.as_ref(), plane);
    }

    /// For the self-test: the roof tile on `plane` nearest fine position `player`, and how many
    /// pieces standing there would hide (the frame after puts the real ones back).
    pub fn roof_check(&mut self, view: &View, player: (i32, i32), plane: usize) -> Option<((usize, usize), usize)> {
        let n = SCENE as i32;
        let (px, py) = (player.0 >> 9, player.1 >> 9);
        let settings = self.ground.settings();
        let tile = (0..n)
            .flat_map(|x| (0..n).map(move |y| (x, y)))
            .filter(|&(x, y)| settings[plane][(x * n + y) as usize] & 4 != 0)
            .min_by_key(|&(x, y)| (x - px).abs().max((y - py).abs()))?;
        self.hide_roofs(view, ((tile.0 << 9) + 256, (tile.1 << 9) + 256), plane);
        Some(((tile.0 as usize, tile.1 as usize), self.scene.hidden()))
    }
}
