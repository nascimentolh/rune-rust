//! Interfaces as the client runs them: components loaded from index 3
//! (`Interface.INTERFACES`), sub-interfaces opened into containers
//! (`client.OPEN_INTERFACES`), client variables, layout and drawing.
//!
//! - `component` — a live component (cache definition + script changes)
//! - `assets`    — scripts, sprites and fonts fetched on demand
//! - `input`     — mouse and keyboard hooks, once per tick (`method11768`)
//! - `drag`      — dragging components (`method12221`, `method2827`)
//! - `layout`    — sizes and positions (`initSizes`, `method4204`, `method16099`, `method8044`)
//! - `render`    — drawing (`IComponentDefinitions.render`)

pub mod assets;
pub mod component;
mod drag;
pub mod input;
mod layout;
mod menu;
mod render;
pub mod social;
pub mod worlds;

use crate::cache::interface::{Component, Kind};
use assets::Assets;
use component::{Comp, ON_LOAD};
use std::collections::HashMap;

/// Player variable changes (`client.anInt7453`, `anIntArray7379`): a running count and the
/// last 32 changed ids. A change is noted when set and recorded once per tick, each variable
/// once (`PulseEvent.processPulseEvents`, before the interfaces' input pass).
#[derive(Clone, Default)]
pub struct VarpChanges {
    pub count: u32,
    pub recent: [i32; 32],
    noted: Vec<i32>,
}

impl VarpChanges {
    fn record(&mut self) {
        for id in std::mem::take(&mut self.noted) {
            self.recent[(self.count & 31) as usize] = id;
            self.count += 1;
        }
    }
}

/// `Loader.JS5_SOURCE_WORLD`.
const JS5_SOURCE_WORLD: i32 = 177;

/// The fixed-size game area (`client.GAME_WIDTH` × `GAME_HEIGHT`, from the entity defaults):
/// the whole client area in fixed mode, and its size before the window is set up.
pub const GAME_WIDTH: i32 = 765;
pub const GAME_HEIGHT: i32 = 503;

/// A component by full id, or one of its created children (`Index.getIComponentDefinitions`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompRef {
    pub id: u32,
    /// -1 for the component itself, else the created child's slot.
    pub slot: i32,
}

impl CompRef {
    pub fn of(id: u32) -> Self {
        CompRef { id, slot: -1 }
    }
}

/// The components of one interface, by index; a created child list has the same shape.
pub type Layer = Vec<Option<Comp>>;

/// The custom top bar (Home / Support / Forums / Clans) of the game's resizable window: 746:239,
/// 50 high, with 746:48 under it (y 50, height "parent − 50").
const GAME_TOP_BAR: (u32, usize) = (746, 239);

pub struct Ui {
    pub assets: Assets,
    interfaces: HashMap<u32, Layer>,
    /// Container id → the interface open inside it.
    subs: HashMap<u32, u32>,
    /// `BASE_WINDOW_ID`.
    root: Option<u32>,
    /// Show the custom top bar of the game's resizable window (`GAME_TOP_BAR`; `server.cfg`
    /// `lobby_top_bar`, as for the lobby's). The reference shows it.
    pub game_top_bar: bool,
    pub prefs: crate::client::preferences::Preferences,
    pub varc: HashMap<i32, i32>,
    /// `GAME_CONNECTION_INFO.worldId`: the cache's world (client parameter 16,
    /// `JS5_SOURCE_WORLD` = 177) until the player picks one in the lobby.
    pub game_world: i32,
    /// `GAME_CONNECTION_INFO.host`: the chosen world's host (`instr6806`).
    pub game_host: String,
    /// `MY_PLAYER.displayName`, from the login reply.
    pub player_name: String,
    /// Packets for the server (id, payload), sent by the connection that is open.
    pub outgoing: Vec<(u32, Vec<u8>)>,
    /// Friends and ignore lists.
    pub social: social::Social,
    /// The lobby's world list.
    pub worlds: worlds::Worlds,
    /// The lobby login reply and when it came (for the membership time left).
    pub lobby: Option<(crate::protocol::login::LobbyDetails, std::time::Instant)>,
    /// Player variables (`PLAYER_VAR_PROVIDER.vars`), set by the server's packets (`set_varp`).
    varp: HashMap<i32, i32>,
    /// Which player variables changed, for the hooks waiting on them.
    varp_changes: VarpChanges,
    pub varc_strings: HashMap<i32, String>,
    /// The components `CC_*` instructions act on (`hookedInterface1` / `hookedInterface2`).
    /// The reference keeps them in the executor, which is reused, so they outlive one script.
    pub active: [Option<CompRef>; 2],
    started: std::time::Instant,
    /// Name and password a script asked to log in with (`instr6707` → `Class155.method2635`).
    pub login_request: Option<(String, String)>,
    /// `Class9.anInt72`: how the last login went, for the window's scripts (`instr6709`):
    /// -2 none, -3 connecting, else the reply code (2 = logged in, -4 = no connection).
    pub login_reply: i32,
    /// The component being dragged (`client.aClass118_7257` and its state).
    drag: Option<drag::Drag>,
    /// The cursor at the last tick, where a dragged component is drawn.
    mouse: (i32, i32),
    /// Where the preferences are saved (`preferences.dat`); `None` for the tools and the
    /// self-test, which must not touch the player's file.
    pub prefs_file: Option<std::path::PathBuf>,
    /// `client.aBool7465`: safe mode was still on at start-up (the last run didn't reach
    /// the login window), so the saved toolkit was reset to 0.
    pub safe_mode_at_start: bool,
    /// The window can time the graphics card for Auto Setup (not the tools, which have no
    /// window to draw in).
    pub can_profile_gpu: bool,
    /// The music player's state (`Class148`).
    pub jukebox: crate::client::music::jukebox::Jukebox,
    /// `BASE_WINDOW_WIDTH` × `BASE_WINDOW_HEIGHT`: the area interfaces are laid out in.
    pub width: i32,
    pub height: i32,
    /// `client.resizeableScreen`: screen size mode 2 (`Class158.windowedMode`).
    pub resizable: bool,
    /// `instr6641` (Play) was run: the app logs in to the world (`Class466.method7777`).
    pub play_requested: bool,
    /// `instr6623` (log out) was run: the app closes the lobby (`CursorIndexLoader.logout`).
    pub logout_requested: bool,
    /// `Class9.anInt106`: how the world login from the lobby went (`method13784` while
    /// `lobbyStage` is 273), for the lobby's scripts (`instr6305`): -2 none, -3 connecting,
    /// else the reply code.
    pub world_login_reply: i32,
    /// A screen size mode a script asked for (`instr6645` → `UID192.method7373`), done by
    /// the window.
    pub mode_request: Option<i32>,
    /// Resize hooks a layout queued (`PENDING_HOOK_REQUESTS`), run by the next tick.
    resize_hooks: Vec<crate::client::cs2::Hook>,
    /// A toolkit 1 score given by the tools (`opengl n` input step) instead of timing.
    pub opengl_score: Option<i32>,
    /// `ClanVarSettingsIndexLoader.anInt2880`: the toolkit the last Auto Setup chose.
    pub auto_setup_toolkit: i32,
    /// `Class188.aBool2378` / `aBool2377` / `aBool2372`, by toolkit (2, 1, 3): it was turned
    /// down after an Auto Setup, so the next one leaves it out.
    pub failed_toolkits: [bool; 4],
}

impl Ui {
    pub fn new(assets: Assets) -> Self {
        Ui {
            assets,
            interfaces: HashMap::new(),
            subs: HashMap::new(),
            root: None,
            game_top_bar: true,
            prefs: Default::default(),
            varc: HashMap::new(),
            game_world: JS5_SOURCE_WORLD,
            game_host: String::new(),
            player_name: String::new(),
            outgoing: Vec::new(),
            social: social::Social::default(),
            worlds: worlds::Worlds::default(),
            lobby: None,
            varp: HashMap::new(),
            varp_changes: VarpChanges::default(),
            varc_strings: HashMap::new(),
            active: [None, None],
            started: std::time::Instant::now(),
            login_request: None,
            login_reply: -2,
            drag: None,
            mouse: (0, 0),
            prefs_file: None,
            safe_mode_at_start: false,
            can_profile_gpu: false,
            jukebox: Default::default(),
            width: GAME_WIDTH,
            height: GAME_HEIGHT,
            resizable: false,
            mode_request: None,
            play_requested: false,
            logout_requested: false,
            world_login_reply: -2,
            resize_hooks: Vec::new(),
            opengl_score: None,
            auto_setup_toolkit: 0,
            failed_toolkits: [false; 4],
        }
    }

    /// `MovingAnimation.isInterfaceLoaded`: fetches and decodes the interface once.
    pub fn load(&mut self, interface: u32) -> bool {
        if self.interfaces.contains_key(&interface) {
            return true;
        }
        let Some(files) = self.assets.archive(3, interface) else { return false };
        // `SkyboxIndexLoader.getInterface`: component n is file n.
        let layer: Result<Layer, _> = files
            .iter()
            .enumerate()
            .map(|(i, f)| f.as_ref().map(|d| Component::decode(interface << 16 | i as u32, d).map(|c| Comp::from_def(&c))).transpose())
            .collect();
        match layer {
            Ok(layer) => {
                self.interfaces.insert(interface, layer);
                true
            }
            Err(e) => {
                eprintln!("interface {interface}: {e}");
                false
            }
        }
    }

    /// `Class383.method6513(1, 1000)`: toolkit 1's score, 1 with no profiling model
    /// (`ENTITY_DEFAULTS.profilingModel == -1`), `None` when it can't run.
    pub fn profile_gpu(&mut self) -> Option<i32> {
        if self.opengl_score.is_some() || !self.can_profile_gpu {
            return self.opengl_score;
        }
        let Some(id) = self.assets.defaults().ok()?.profiling_model else { return Some(1) };
        let mesh = self.assets.mesh(id)?;
        Some(crate::client::gpu::profile(&mesh, 1000, (self.width, self.height)))
    }

    /// `ClanVarDefinitions.method6825`: the full-screen sizes for this monitor.
    pub fn full_screen_modes(&self) -> Vec<(i32, i32)> {
        crate::client::display::modes(crate::client::display::monitor(), self.prefs.max_screen_size)
    }

    /// Game state 0 (`QuickChatMessage.method6161`): the lobby window from the entity defaults
    /// (opcode 6) becomes the root, with its on-load scripts.
    /// `top_bar` false hides the top bar (component 1, not in the reference: see `Settings`).
    pub fn open_lobby(&mut self, details: &crate::protocol::login::LobbyDetails, top_bar: bool) -> Result<(), String> {
        self.player_name = details.name.clone();
        self.lobby = Some((details.clone(), std::time::Instant::now()));
        self.unload_root();
        let id = self.assets.defaults()?.lobby_window.ok_or("no lobby window in the entity defaults")?;
        if !self.load(id) {
            return Err(format!("lobby interface {id} isn't in the cache"));
        }
        if !top_bar {
            self.hide_top_bar(id, 1);
        }
        self.open_root(id);
        Ok(())
    }

    /// Hides a window's top bar (the lobby's component 1, `GAME_TOP_BAR`) and moves what sat
    /// under it up by its height: top-level components placed at its height (y mode 0), and
    /// those sized "parent minus n" (height mode 1) grow by it.
    fn hide_top_bar(&mut self, interface: u32, component: usize) {
        let Some(layer) = self.interfaces.get_mut(&interface) else { return };
        let Some(bar) = layer.get_mut(component).and_then(Option::as_mut) else { return };
        bar.hidden = true;
        let height = bar.base_height;
        for c in layer.iter_mut().flatten().filter(|c| c.parent.is_none() && c.y_mode == 0 && c.base_y == height) {
            c.base_y = 0;
            if c.height_mode == 1 && c.base_height >= height {
                c.base_height -= height;
            }
        }
    }

    /// A player variable (`PLAYER_VAR_PROVIDER.vars`; 0 until the server sets it).
    pub fn varp(&self, id: i32) -> i32 {
        self.varp.get(&id).copied().unwrap_or(0)
    }

    /// `setVarp` from a server packet: stores it and notes the change for the hooks.
    pub fn set_varp(&mut self, id: i32, value: i32) {
        self.varp.insert(id, value);
        if !self.varp_changes.noted.contains(&id) {
            self.varp_changes.noted.push(id);
        }
    }

    /// `PlayerVarProvider.getVarBit`: bits `start..=end` of the variable the varbit names
    /// (`BIT_MASKS[end − start]` = 2^(end − start + 1) − 1, −1 for all 32).
    pub fn varbit(&mut self, id: i32) -> Result<i32, String> {
        let bit = self.assets.varbit(id)?;
        let mask = 2i32.wrapping_shl((bit.end - bit.start) as u32).wrapping_sub(1);
        Ok(self.varp(bit.base) >> bit.start & mask)
    }

    /// `ObjectDefinition.pulseSounds` → `MaterialProp33.method15420`, `ProcessorSpecs.method7730`.
    pub fn tick_music(&mut self) {
        let stereo = self.prefs.mono_stereo == 1;
        let assets = &mut self.assets;
        self.jukebox.tick(stereo, &mut |index, track| assets.song(index, track));
    }

    /// `GameState.setGameState` into the login screen or lobby: the login music
    /// (`Class260.anInt3223 = anInt3228`) at the login music volume, or silence when that is 0.
    pub fn enter_login_screen(&mut self) {
        let Some(track) = self.assets.login_track() else { return };
        self.jukebox.song = track as i32;
        match self.prefs.sub13_volume {
            0 => self.jukebox.stop(),
            v => self.jukebox.start(track, v, 2),
        }
    }

    /// `CursorIndexLoader.logout(false)` from the lobby, once the app has closed its
    /// connections: no login under way (`JS5StandardRequester.method5558`: both reply codes
    /// back to -2), the music fades out and is forgotten (`Class358.method6240`,
    /// `Class260.anInt3223 = -1`), then game state 5 opens the login window again
    /// (`Class348.method6175`). The lobby and the login screen are in the same group of states,
    /// so the login music doesn't start again (`GameState.setGameState`).
    pub fn log_out(&mut self) -> Result<(), String> {
        (self.login_reply, self.world_login_reply) = (-2, -2);
        (self.play_requested, self.logout_requested) = (false, false);
        self.jukebox.stop();
        self.jukebox.song = -1;
        self.lobby = None;
        self.drag = None;
        self.unload_root();
        let id = self.assets.login_window()?;
        if !self.load(id) {
            return Err(format!("interface {id} isn't in the cache"));
        }
        self.open_root(id);
        Ok(())
    }

    /// A new root window replaces the old one (`QuickChatMessage.method6161`,
    /// `Class348.method6175`): the old root is unloaded (`Class337.unloadInterface`) and so is
    /// every interface open inside it (`Class351.closeChildren(.., true, ..)`), so they come
    /// back fresh when opened again.
    fn unload_root(&mut self) {
        if let Some(old) = self.root.take() {
            self.interfaces.remove(&old);
        }
        for (_, sub) in std::mem::take(&mut self.subs) {
            self.interfaces.remove(&sub);
        }
    }

    /// `client.BASE_WINDOW_ID`: the interface open as the root window.
    pub fn root_id(&self) -> Option<u32> {
        self.root
    }

    /// `Class190.savePreferences`.
    pub fn save_prefs(&self) {
        if let Some(path) = &self.prefs_file {
            self.prefs.save(path);
        }
    }

    /// Start-up with the saved preferences (`FixedTileStrategy.method12791`, then
    /// `IndexLoaders`: safe mode on for the run, toolkit 0 if it was already on), and
    /// once the login window is up safe mode off again, saved (`client.java:2165`).
    pub fn use_saved_prefs(&mut self, path: std::path::PathBuf) {
        self.prefs = crate::client::preferences::Preferences::load(&path);
        self.safe_mode_at_start = self.prefs.safe_mode == 1;
        if self.safe_mode_at_start {
            self.prefs.toolkit = 0;
        }
        self.prefs.safe_mode = 0;
        self.prefs_file = Some(path);
        self.save_prefs();
    }

    /// `Index.getIComponentDefinitions` (loading the interface like `getDefs`).
    pub fn comp(&mut self, r: CompRef) -> Option<&mut Comp> {
        let interface = r.id >> 16;
        if !self.load(interface) {
            return None;
        }
        let c = self.interfaces.get_mut(&interface)?.get_mut((r.id & 0xffff) as usize)?.as_mut()?;
        if r.slot == -1 {
            Some(c)
        } else {
            c.children.get_mut(usize::try_from(r.slot).ok()?)?.as_mut()
        }
    }

    /// Opens the top-level window (`Class348.method6175`): lays it out, then runs the
    /// on-load script of every component in order (`Connection.runIComponentScripts`).
    pub fn open_root(&mut self, interface: u32) {
        self.subs.clear();
        self.root = Some(interface);
        let (w, h) = (self.width, self.height);
        self.layout_interface(interface, w, h);
        self.run_on_load(interface);
    }

    /// `IF_OPENTOP` (`client.BASE_WINDOW_ID`): the window becomes the root (`ClipFlagMap.method6007`
    /// loads it), is laid out (`Class516.method8867`) and its on-load scripts run
    /// (`Class150.method2582`); what is open in it stays.
    pub fn open_top(&mut self, interface: u32) {
        if !self.load(interface) {
            eprintln!("window {interface} isn't in the cache");
            return;
        }
        if !self.game_top_bar && interface == GAME_TOP_BAR.0 {
            self.hide_top_bar(interface, GAME_TOP_BAR.1);
        }
        self.root = Some(interface);
        let (w, h) = (self.width, self.height);
        self.layout_interface(interface, w, h);
        self.run_on_load(interface);
    }

    /// `IdentiKitIndexLoader.openSub` as `IF_OPENSUB` calls it: as `open_sub`, then the new
    /// interface's on-load scripts run.
    pub fn open_sub_from_server(&mut self, parent: u32, interface: u32) {
        self.open_sub(parent, interface);
        self.run_on_load(interface);
    }

    /// Leaving the lobby for the world (game state 30): nothing is shown until the server
    /// opens the game's window.
    pub fn close_root(&mut self) {
        self.unload_root();
    }

    fn run_on_load(&mut self, interface: u32) {
        if !self.load(interface) {
            return;
        }
        let count = self.interfaces[&interface].len();
        for i in 0..count {
            let r = CompRef::of(interface << 16 | i as u32);
            let Some(args) = self.comp(r).and_then(|c| c.hook(ON_LOAD).cloned()) else { continue };
            crate::client::cs2::run_hook(self, &crate::client::cs2::Hook { source: Some(r), args, ..Default::default() }, 2_000_000);
        }
    }

    /// `IdentiKitIndexLoader.openSub` as `IF_OPENSUBCLIENT` calls it: the new interface's
    /// on-load scripts are not run.
    pub fn open_sub(&mut self, parent: u32, interface: u32) {
        if let Some(open) = self.subs.get(&parent).copied() {
            self.close_sub(parent, open != interface);
        }
        self.subs.insert(parent, interface);
        self.load(interface);
        self.layout_contents(CompRef::of(parent));
    }

    /// `Class351.closeChildren`: closes what is open in `parent`, and everything open inside it.
    pub fn close_sub(&mut self, parent: u32, unload: bool) {
        let Some(interface) = self.subs.remove(&parent) else { return };
        if unload {
            self.interfaces.remove(&interface);
        }
        let nested: Vec<u32> = self.subs.keys().copied().filter(|p| p >> 16 == interface).collect();
        for p in nested {
            self.close_sub(p, true);
        }
    }

    /// `CYCLES_20MS`: client ticks of 20 ms since start.
    pub fn clock(&self) -> i32 {
        (self.started.elapsed().as_millis() / 20) as i32
    }

    /// Draws the root window.
    pub fn draw(&mut self, raster: &mut crate::client::raster::Raster) {
        if let Some(root) = self.root {
            render::draw(self, root, raster);
        }
    }

    /// Where the shown text component reading `text` is drawn (x, y, width, height), placed
    /// as `render` places it (for the self-test's clicks).
    pub fn text_box(&self, text: &str) -> Option<(i32, i32, i32, i32)> {
        let layer = self.interfaces.get(&self.root?)?;
        self.find_text(layer, None, (0, 0), text)
    }

    fn find_text(&self, layer: &Layer, parent: Option<u32>, (ox, oy): (i32, i32), text: &str) -> Option<(i32, i32, i32, i32)> {
        layer.iter().flatten().filter(|c| c.parent == parent && !c.hidden).find_map(|c| {
            let (x, y) = (c.x + ox, c.y + oy);
            match c.kind {
                Kind::Text if c.text == text => Some((x, y, c.width, c.height)),
                Kind::Container => {
                    let inner = (x - c.scroll_x, y - c.scroll_y);
                    self.find_text(layer, Some(c.id), inner, text).or_else(|| self.find_text(&c.children, Some(c.id), inner, text)).or_else(
                        || self.subs.get(&c.id).and_then(|i| self.interfaces.get(i)).and_then(|s| self.find_text(s, None, (x, y), text)),
                    )
                }
                _ => None,
            }
        })
    }
}
