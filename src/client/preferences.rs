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
    /// `antiAliasingDefault`: the saved anti-aliasing.
    pub anti_aliasing_default: i32,
    /// `aPreference_Sub4_8223` (anti-aliasing), default 0.
    pub anti_aliasing: i32,
    pub bloom: i32,
    pub brightness: i32,
    /// `buildArea` (`MapSize` index, 0 = 104 tiles).
    pub build_area: i32,
    /// `aPreference_Sub16_8198`.
    pub sub16: i32,
    /// `Preference_Sub26`.
    pub flickering: i32,
    pub fog: i32,
    /// `Preference_Sub17`.
    pub ground_blending: i32,
    pub ground_decor: i32,
    pub idle_animations: i32,
    pub light_detail: i32,
    pub scenery_shadows: i32,
    /// `toolkitDefault` (`Preference_Sub29`).
    pub toolkit_default: i32,
    /// `particles` (`Preference_Sub20`): 2 with 245 MB or more.
    pub particles: i32,
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
    pub custom_cursors: i32,
    pub graphics: i32,
    /// `CPUMaxMemoryPreference`: 4 with more than one processor, else 2.
    pub cpu: i32,
    /// `aPreference_Sub11_8217`.
    pub sub11: i32,
    /// `safeMode`: on while the client starts, off once the login window is up.
    pub safe_mode: i32,
    /// `aPreference_Sub3_8199`.
    pub sub3: i32,
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
            anti_aliasing_default: 0,
            anti_aliasing: 0,
            bloom: 0,
            brightness: 3,
            build_area: 0,
            sub16: 0,
            flickering: 1,
            fog: 1,
            ground_blending: 1,
            ground_decor: 1,
            idle_animations: 1,
            light_detail: 1,
            scenery_shadows: 2,
            toolkit_default: 0,
            particles: 2,
            remove_roofs: 2,
            max_screen_size: 1,
            skyboxes: 1,
            character_shadows: 1,
            textures: 1,
            toolkit: 2,
            current_toolkit: 0,
            water: 1,
            screen_size: 2,
            custom_cursors: 1,
            graphics: 0,
            cpu: if std::thread::available_parallelism().map_or(1, |n| n.get()) > 1 { 4 } else { 2 },
            sub11: 0,
            safe_mode: 0,
            sub3: 0,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pref {
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
    AntiAliasingDefault,
    Bloom,
    Skyboxes,
    MaxScreenSize,
    Graphics,
    Brightness,
    BuildArea,
    Particles,
    ToolkitDefault,
    Cpu,
}

impl Preferences {
    fn slot(&mut self, pref: Pref) -> &mut i32 {
        match pref {
            Pref::RemoveRoofs => &mut self.remove_roofs,
            Pref::GroundDecor => &mut self.ground_decor,
            Pref::GroundBlending => &mut self.ground_blending,
            Pref::IdleAnimations => &mut self.idle_animations,
            Pref::Flickering => &mut self.flickering,
            Pref::CharacterShadows => &mut self.character_shadows,
            Pref::SceneryShadows => &mut self.scenery_shadows,
            Pref::Textures => &mut self.textures,
            Pref::LightDetail => &mut self.light_detail,
            Pref::Water => &mut self.water,
            Pref::Fog => &mut self.fog,
            Pref::AntiAliasing => &mut self.anti_aliasing,
            Pref::AntiAliasingDefault => &mut self.anti_aliasing_default,
            Pref::Bloom => &mut self.bloom,
            Pref::Skyboxes => &mut self.skyboxes,
            Pref::MaxScreenSize => &mut self.max_screen_size,
            Pref::Graphics => &mut self.graphics,
            Pref::Brightness => &mut self.brightness,
            Pref::BuildArea => &mut self.build_area,
            Pref::Particles => &mut self.particles,
            Pref::ToolkitDefault => &mut self.toolkit_default,
            Pref::Cpu => &mut self.cpu,
        }
    }

    /// Each preference's `checkValid`: 1 fine, 2 allowed but something else gets turned off
    /// (the options panel warns), 3 refused. The client has 512 MB or more (`getMaxMemory`),
    /// so only the 168-tile build area needing 500 MB is refused for memory — never here.
    pub fn check_valid(&self, pref: Pref, v: i32) -> i32 {
        let effects = toolkit_has_effects(self.current_toolkit);
        match pref {
            Pref::AntiAliasing | Pref::AntiAliasingDefault | Pref::Bloom | Pref::Skyboxes if !effects => 3,
            Pref::Fog if v != 0 && self.toolkit_default == 2 => 3,
            Pref::Fog | Pref::Water | Pref::Textures if v != 0 && self.ground_blending != 1 => 2,
            Pref::GroundBlending if v == 0 && (self.fog == 1 || self.textures == 1 || self.water > 0) => 2,
            Pref::SceneryShadows if self.textures == 0 => 3,
            Pref::ToolkitDefault if v == 2 => 2,
            _ => 1,
        }
    }

    /// The `method12646` … `method12923` "can be changed" checks (`DETAILCANMOD_*`). Anti-
    /// aliasing and bloom also need the renderer to have them (`method8405`, `method8403`),
    /// which is a toolkit with effects here.
    pub fn can_modify(&self, pref: Pref) -> bool {
        match pref {
            Pref::AntiAliasing | Pref::AntiAliasingDefault | Pref::Bloom | Pref::Skyboxes => toolkit_has_effects(self.current_toolkit),
            Pref::SceneryShadows => self.textures != 0,
            _ => true,
        }
    }

    /// `GamePreferences.setValue`: `setPref` refuses a value its `checkValid` rates 3, then
    /// `method13502` checks every preference again.
    pub fn set(&mut self, pref: Pref, v: i32) {
        if self.check_valid(pref, v) != 3 {
            *self.slot(pref) = v;
        }
        self.check();
    }

    /// Auto Setup (`ApplyHitmarkCutsceneAction.method14641`). `opengl` is toolkit 1's score
    /// (`gpu::profile`), `None` where it can't start; toolkits 2 and 3 never start here (-1).
    /// With no score the CPU decides (`software_setup`). Otherwise the score × 1.1 picks the
    /// level (`FontRenderer_Sub1.method13782`: > 20077 → 4, > 10021 → 3, > 5002 → 2, else 1)
    /// on toolkit 1, already running since the timing started it (`Class231.method3914` sets
    /// only `currentToolkit`), so its effects count. The saved toolkit stays until the
    /// player keeps the change ("Yes" → `instr6922`).
    pub fn auto_setup(&mut self, opengl: Option<i32>) -> i32 {
        let Some(score) = opengl else { return self.software_setup() };
        let score = (score as f32 * 1.1) as i32;
        self.current_toolkit = 1;
        let level = match score {
            s if s > 20077 => 4,
            s if s > 10021 => 3,
            s if s > 5002 => 2,
            _ => 1,
        };
        self.apply_preset(level);
        level
    }

    /// `Node_Sub17_Sub1.method15402`: start toolkit `t`. Only 0 (software) and 1 (OpenGL,
    /// here macroquad's) start in this client; any other fails and the one running stays.
    pub fn start_toolkit(&mut self, t: i32) {
        if matches!(t, 0 | 1) {
            self.current_toolkit = t;
        }
    }

    /// `Class115.setBloom`: bloom only starts on a toolkit with effects; true when the
    /// preference changed (the reference then saves).
    pub fn set_bloom(&mut self, on: bool) -> bool {
        let now = i32::from(on && toolkit_has_effects(self.current_toolkit));
        let changed = now != self.bloom;
        self.bloom = now;
        changed
    }

    /// `ServerEnvironment.method8313`, Auto Setup with no hardware toolkit: picks a detail
    /// level from the CPU profile (≤ 104 ms → 4, ≤ 504 → 3, ≤ 1002 → 2, else 1; the client has
    /// 96 MB or more), applies its preset and returns the level. Then the software toolkit:
    /// when another one runs, toolkit 0 is saved and started.
    fn software_setup(&mut self) -> i32 {
        let level = match cpu_profile() {
            ms if ms <= 104 => 4,
            ms if ms <= 504 => 3,
            ms if ms <= 1002 => 2,
            _ => 1,
        };
        self.apply_preset(level);
        if self.current_toolkit != 0 {
            self.toolkit = 0;
            self.start_toolkit(0);
        }
        level
    }

    /// One of the four detail presets (also the Options panel's MIN / LOW / MID / HIGH:
    /// `AUTOSETUP_SETMIN` … `AUTOSETUP_SETHIGH`), with their particles (2, 1, 0, 0) and build
    /// area (104 tiles).
    pub fn apply_preset(&mut self, level: i32) {
        for (pref, v) in preset(level) {
            self.set(pref, v);
        }
        self.particles = match level {
            4 => 2,
            3 => 1,
            _ => 0,
        };
        self.build_area = 0;
    }

    /// `GamePreferences.encode`: version 27, then one byte per preference.
    pub fn encode(&self) -> Vec<u8> {
        [
            VERSION,
            self.anti_aliasing_default,
            self.bloom,
            self.brightness,
            self.build_area,
            self.sub16,
            self.flickering,
            self.fog,
            self.ground_blending,
            self.ground_decor,
            self.idle_animations,
            self.light_detail,
            self.scenery_shadows,
            self.toolkit_default,
            self.particles,
            self.remove_roofs,
            self.max_screen_size,
            self.skyboxes,
            self.character_shadows,
            self.textures,
            self.toolkit,
            0,
            self.water,
            self.screen_size,
            self.custom_cursors,
            self.graphics,
            self.cpu,
            self.sub11,
            self.safe_mode,
            self.sub3,
            self.sound_volume,
            self.ambient_volume,
            self.voice_volume,
            self.music_volume,
            self.sub13_volume,
            self.mono_stereo,
        ]
        .iter()
        .map(|&v| v as u8)
        .collect()
    }

    /// `GamePreferences(ByteBuf)` → `decode`: versions 23 to 27 (older ones, written in a
    /// layout not ported, and newer ones give the defaults), then `method13502` checks them.
    /// A file cut short keeps the defaults for what is missing (the reference throws and
    /// gives up on all of them; this file is only ever written whole).
    ///
    /// Kept from the reference: `encode` writes the custom cursors byte but `decode` doesn't
    /// read it (`customCursors = new CustomCursorsPreference(this)`), so from graphics on each
    /// preference reads the byte of the one before it (music volume gets the voice-over one).
    pub fn decode(data: &[u8]) -> Self {
        let mut p = Preferences::default();
        let Some((&version, rest)) = data.split_first() else { return p };
        if !(23..=VERSION).contains(&(version as i32)) {
            return p;
        }
        let version = version as i32;
        let mut bytes = rest.iter().map(|&b| b as i32);
        let mut next = |slot: &mut i32| {
            if let Some(v) = bytes.next() {
                *slot = v;
            }
        };
        next(&mut p.anti_aliasing_default);
        p.anti_aliasing = p.anti_aliasing_default;
        next(&mut p.bloom);
        next(&mut p.brightness);
        next(&mut p.build_area);
        if version >= 27 {
            next(&mut p.sub16);
        }
        for slot in [
            &mut p.flickering,
            &mut p.fog,
            &mut p.ground_blending,
            &mut p.ground_decor,
            &mut p.idle_animations,
            &mut p.light_detail,
            &mut p.scenery_shadows,
        ] {
            next(slot);
        }
        if version >= 24 {
            next(&mut p.toolkit_default);
        }
        next(&mut p.particles);
        next(&mut p.remove_roofs);
        next(&mut p.max_screen_size);
        if version >= 25 {
            next(&mut p.skyboxes);
        }
        next(&mut p.character_shadows);
        if version <= 25 {
            next(&mut 0);
        }
        next(&mut p.textures);
        next(&mut p.toolkit);
        p.current_toolkit = p.toolkit;
        next(&mut 0);
        for slot in [&mut p.water, &mut p.screen_size, &mut p.graphics, &mut p.cpu, &mut p.sub11, &mut p.safe_mode] {
            next(slot);
        }
        if version >= 26 {
            next(&mut p.sub3);
        }
        for slot in
            [&mut p.sound_volume, &mut p.ambient_volume, &mut p.voice_volume, &mut p.music_volume, &mut p.sub13_volume, &mut p.mono_stereo]
        {
            next(slot);
        }
        p.check();
        p
    }

    /// `GamePreferences.method13502`: each preference back to its default when out of range,
    /// in the reference's order. Water, CPU and the volumes are checked with `< 0 && > n`,
    /// which is never true, so any value stays — kept as the reference has it. The
    /// anti-aliasing and bloom checks against the started toolkit are in `set`.
    fn check(&mut self) {
        let d = Preferences::default();
        let fix = |v: &mut i32, valid: std::ops::RangeInclusive<i32>, default: i32| {
            if !valid.contains(v) {
                *v = default;
            }
        };
        fix(&mut self.anti_aliasing_default, 0..=2, 0);
        fix(&mut self.anti_aliasing, 0..=2, 0);
        fix(&mut self.bloom, 0..=1, 0);
        fix(&mut self.brightness, 0..=4, d.brightness);
        // 512 MB or more: only the range check of `buildArea` applies.
        fix(&mut self.build_area, 0..=4, 0);
        fix(&mut self.sub16, 0..=2, 0);
        fix(&mut self.flickering, 0..=1, d.flickering);
        if self.fog != 0 && (self.ground_blending != 1 || self.toolkit_default == 2) {
            self.fog = 0;
        }
        fix(&mut self.fog, 0..=1, d.fog);
        fix(&mut self.ground_blending, 0..=1, d.ground_blending);
        fix(&mut self.ground_decor, 0..=1, d.ground_decor);
        fix(&mut self.idle_animations, 0..=2, d.idle_animations);
        fix(&mut self.light_detail, 0..=1, d.light_detail);
        if self.textures == 0 {
            self.scenery_shadows = 0;
        }
        fix(&mut self.scenery_shadows, 0..=2, d.scenery_shadows);
        fix(&mut self.toolkit_default, 0..=2, 0);
        fix(&mut self.particles, 0..=2, d.particles);
        if self.toolkit_default == 2 && self.remove_roofs == 2 {
            self.remove_roofs = 1;
        }
        fix(&mut self.remove_roofs, 0..=2, d.remove_roofs);
        fix(&mut self.max_screen_size, 0..=2, d.max_screen_size);
        fix(&mut self.skyboxes, 0..=1, d.skyboxes);
        fix(&mut self.character_shadows, 0..=1, d.character_shadows);
        fix(&mut self.textures, 0..=1, d.textures);
        fix(&mut self.toolkit, 0..=5, d.toolkit);
        fix(&mut self.screen_size, 1..=3, d.screen_size);
        fix(&mut self.custom_cursors, 0..=1, 1);
        fix(&mut self.graphics, 0..=4, 0);
        fix(&mut self.safe_mode, 0..=1, 0);
        // `Preference_Sub3` is only on with toolkit 3 or 5 and two processors or more.
        let processors = std::thread::available_parallelism().map_or(1, |n| n.get());
        if (self.toolkit != 3 && self.toolkit != 5) || processors < 2 {
            self.sub3 = 0;
        }
        fix(&mut self.sub3, 0..=1, 0);
        fix(&mut self.mono_stereo, 0..=1, d.mono_stereo);
    }

    /// `FixedTileStrategy.method12791`: the saved preferences, or the defaults.
    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read(path).map(|data| Preferences::decode(&data)).unwrap_or_default()
    }

    /// `Class190.savePreferences`: errors are ignored, as there.
    pub fn save(&self, path: &std::path::Path) {
        let _ = std::fs::write(path, self.encode());
    }
}

/// `GamePreferences.encode`'s version byte.
const VERSION: i32 = 27;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_preferences_read_back() {
        let mut p = Preferences::default();
        p.apply_preset(2);
        (p.music_volume, p.mono_stereo, p.voice_volume) = (0, 0, 50);
        let bytes = p.encode();
        assert_eq!((bytes.len(), bytes[0]), (36, 27));
        let q = Preferences::decode(&bytes);
        assert_eq!(q.encode()[..24], bytes[..24]);
        // The reference reads back one byte fewer than it writes (no custom cursors byte), so
        // from graphics on each preference gets the byte of the one before it.
        assert_eq!((q.graphics, q.cpu, q.music_volume, q.sub13_volume), (p.custom_cursors, p.graphics, p.voice_volume, p.music_volume));
        assert_eq!(q.mono_stereo, 1); // got 127 (the login music volume): out of range
                                      // Out of range goes back to the default; the volumes are never checked.
        let mut bad = bytes.clone();
        (bad[3], bad[32]) = (9, 200);
        let q = Preferences::decode(&bad);
        assert_eq!((q.brightness, q.music_volume), (3, 200));
        assert_eq!(Preferences::decode(&[28]).encode(), Preferences::default().encode());
    }

    #[test]
    fn a_fast_card_gets_high_detail_and_opengl() {
        let mut p = Preferences::default();
        assert_eq!(p.auto_setup(Some(18_300)), 4); // 18 300 × 1.1 = 20 130 > 20 077
        assert_eq!((p.toolkit, p.current_toolkit, p.skyboxes), (2, 1, 1));
        let mut p = Preferences::default();
        assert_eq!(p.auto_setup(Some(18_200)), 3); // 20 020
        assert_eq!(Preferences::default().auto_setup(Some(1)), 1);
    }

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
