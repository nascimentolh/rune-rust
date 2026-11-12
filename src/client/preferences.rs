//! The game preferences scripts read (`GamePreferences`, `Class393.preferences`), as a fresh
//! install has them: no saved preferences file, so every preference takes its
//! `getDefaultValue()` (`GamePreferences(Game)` → `method13497(true)`), for the game
//! `Game.darkan` (id 0) on a machine with several processors and 512 MB or more.
//!
//! Then start-up (`IndexLoaders` stage after the tips) turns safe mode on for the next run
//! and starts the saved toolkit, 2; this client draws with the reference's software
//! (Java) toolkit, which is where the reference lands when toolkit 2 fails to start
//! (`Node_Sub17_Sub1`: `currentToolkit` = 0, `aBool7171` = true).

/// `Class60.method1166`: the toolkits with anti-aliasing, bloom and skyboxes.
pub fn toolkit_has_effects(toolkit: i32) -> bool {
    matches!(toolkit, 1 | 3 | 5)
}

pub struct Preferences {
    /// `aPreference_Sub4_8223` (anti-aliasing), default 0.
    pub anti_aliasing: i32,
    pub bloom: i32,
    pub brightness: i32,
    /// `Preference_Sub26`.
    pub flickering: i32,
    pub fog: i32,
    /// `Preference_Sub17`.
    pub ground_blending: i32,
    pub ground_decor: i32,
    pub idle_animations: i32,
    pub light_detail: i32,
    pub scenery_shadows: i32,
    /// `Preference_Sub27`.
    pub remove_roofs: i32,
    /// `Preference_Sub7`: 1, as the toolkit at construction (0) is in `Class520.method11162`.
    pub max_screen_size: i32,
    /// `Preference_Sub14`.
    pub skyboxes: i32,
    /// `Preference_Sub28`.
    pub character_shadows: i32,
    pub textures: i32,
    /// `toolKit`: the saved choice.
    pub toolkit: i32,
    /// `currentToolkit`: the one drawing.
    pub current_toolkit: i32,
    pub water: i32,
    /// 2 (not ARM).
    pub screen_size: i32,
    pub graphics: i32,
    /// `CPUMaxMemoryPreference`: 4 with more than one processor, else 2.
    pub cpu: i32,
    /// `aPreference_Sub11_8217`.
    pub sub11: i32,
    pub sound_volume: i32,
    pub ambient_volume: i32,
    pub voice_volume: i32,
    pub music_volume: i32,
    /// `aPreference_Sub13_8229`.
    pub sub13_volume: i32,
    pub mono_stereo: i32,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            anti_aliasing: 0,
            bloom: 0,
            brightness: 3,
            flickering: 1,
            fog: 1,
            ground_blending: 1,
            ground_decor: 1,
            idle_animations: 1,
            light_detail: 1,
            scenery_shadows: 2,
            remove_roofs: 2,
            max_screen_size: 1,
            skyboxes: 1,
            character_shadows: 1,
            textures: 1,
            toolkit: 2,
            current_toolkit: 0,
            water: 1,
            screen_size: 2,
            graphics: 0,
            cpu: if std::thread::available_parallelism().map_or(1, |n| n.get()) > 1 { 4 } else { 2 },
            sub11: 0,
            sound_volume: 127,
            ambient_volume: 127,
            voice_volume: 127,
            music_volume: 127,
            sub13_volume: 127,
            mono_stereo: 1,
        }
    }
}

/// `Class241.getCPUProfile`: milliseconds to draw the same triangle 10 000 times with the
/// software toolkit into a 100 × 100 canvas.
pub fn cpu_profile() -> i32 {
    let mut r = super::raster::Raster::new(100, 100);
    let started = std::time::Instant::now();
    for _ in 0..10_000 {
        r.fill_triangle((10, 5), (50, 75), (90, 15), 0xffff_0000);
    }
    started.elapsed().as_millis() as i32
}

/// The graphics presets of `method8313`, by detail level 4 (`EquipmentDefaults.method11248`),
/// 3 (`Node_Sub40.method13300`), 2 (`Class119.method2077`) and 1 (`MaterialProp14.method15393`),
/// as (preference, value) in the order they are set. Particles and build area are set too but
/// nothing here reads them.
fn preset(level: i32) -> Vec<(Pref, i32)> {
    use Pref::*;
    let (roofs, decor, blend, flags, scenery, textures, water, fog, sky, max_screen) = match level {
        4 => (2, 1, 1, 1, 2, 1, 2, 1, 1, 0),
        3 => (2, 1, 1, 1, 1, 1, 0, 1, 1, 1),
        2 => (1, 1, 1, 0, 0, 0, 0, 0, 0, 2),
        _ => (1, 0, 0, 0, 0, 0, 0, 0, 0, 2),
    };
    let mut steps = vec![(RemoveRoofs, roofs), (GroundDecor, decor)];
    if level == 1 {
        // `method15393` sets fog before ground blending; the others after water.
        steps.push((Fog, 0));
    }
    steps.extend([(GroundBlending, blend), (IdleAnimations, flags), (Flickering, flags), (CharacterShadows, flags)]);
    // Levels 2 and 1 set scenery shadows before textures; 4 and 3 after.
    if level <= 2 {
        steps.extend([(SceneryShadows, scenery), (Textures, textures)]);
    } else {
        steps.extend([(Textures, textures), (SceneryShadows, scenery)]);
    }
    steps.extend([(LightDetail, flags), (Water, water)]);
    if level != 1 {
        steps.push((Fog, fog));
    }
    steps.extend([(AntiAliasing, 0), (Bloom, 0), (Skyboxes, sky), (MaxScreenSize, max_screen), (Graphics, level)]);
    steps
}

/// The preferences `setValue` can change here.
#[derive(Clone, Copy)]
enum Pref {
    RemoveRoofs,
    GroundDecor,
    GroundBlending,
    IdleAnimations,
    Flickering,
    CharacterShadows,
    SceneryShadows,
    Textures,
    LightDetail,
    Water,
    Fog,
    AntiAliasing,
    Bloom,
    Skyboxes,
    MaxScreenSize,
    Graphics,
}

impl Preferences {
    /// `GamePreferences.setValue`: `setPref` refuses a value its `checkValid` rates 3, then
    /// `method13502` re-checks the preferences that depend on others.
    fn set(&mut self, pref: Pref, v: i32) {
        let effects = toolkit_has_effects(self.current_toolkit);
        let (slot, refused) = match pref {
            Pref::RemoveRoofs => (&mut self.remove_roofs, false),
            Pref::GroundDecor => (&mut self.ground_decor, false),
            Pref::GroundBlending => (&mut self.ground_blending, false),
            Pref::IdleAnimations => (&mut self.idle_animations, false),
            Pref::Flickering => (&mut self.flickering, false),
            Pref::CharacterShadows => (&mut self.character_shadows, false),
            Pref::SceneryShadows => (&mut self.scenery_shadows, self.textures == 0),
            Pref::Textures => (&mut self.textures, false),
            Pref::LightDetail => (&mut self.light_detail, false),
            Pref::Water => (&mut self.water, false),
            Pref::Fog => (&mut self.fog, false),
            Pref::AntiAliasing => (&mut self.anti_aliasing, !effects),
            Pref::Bloom => (&mut self.bloom, !effects),
            Pref::Skyboxes => (&mut self.skyboxes, !effects),
            Pref::MaxScreenSize => (&mut self.max_screen_size, false),
            Pref::Graphics => (&mut self.graphics, false),
        };
        if !refused {
            *slot = v;
        }
        // `FogPreference.method13415`, `SceneryShadowPreference.method12627`.
        if self.fog != 0 && self.ground_blending != 1 {
            self.fog = 0;
        }
        if self.textures == 0 {
            self.scenery_shadows = 0;
        }
    }

    /// Auto Setup with no hardware toolkit (`ApplyHitmarkCutsceneAction.method14641` finds
    /// none, so `ServerEnvironment.method8313`): picks a detail level from the CPU profile
    /// (≤ 104 ms → 4, ≤ 504 → 3, ≤ 1002 → 2, else 1; the client has 96 MB or more), applies
    /// its preset and returns the level. The software toolkit stays.
    pub fn auto_setup(&mut self) -> i32 {
        let level = match cpu_profile() {
            ms if ms <= 104 => 4,
            ms if ms <= 504 => 3,
            ms if ms <= 1002 => 2,
            _ => 1,
        };
        self.apply_preset(level);
        level
    }

    /// One of the four detail presets (also the Options panel's MIN / LOW / MID / HIGH:
    /// `AUTOSETUP_SETMIN` … `AUTOSETUP_SETHIGH`).
    pub fn apply_preset(&mut self, level: i32) {
        for (pref, v) in preset(level) {
            self.set(pref, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_follow_the_reference() {
        let mut p = Preferences::default();
        p.apply_preset(1);
        assert_eq!((p.textures, p.scenery_shadows, p.fog, p.ground_blending, p.graphics, p.max_screen_size), (0, 0, 0, 0, 1, 2));
        // The software toolkit has no skyboxes, so their setting is refused.
        assert_eq!(p.skyboxes, 1);
        let mut p = Preferences::default();
        p.apply_preset(4);
        assert_eq!((p.water, p.scenery_shadows, p.light_detail, p.graphics, p.max_screen_size), (2, 2, 1, 4, 0));
    }
}
