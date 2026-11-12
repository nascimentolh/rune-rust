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
