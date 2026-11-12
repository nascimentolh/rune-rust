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
mod render;

use crate::cache::interface::Component;
use assets::Assets;
use component::{Comp, ON_LOAD};
use std::collections::HashMap;

/// The fixed-size client area (`BASE_WINDOW_WIDTH` × `BASE_WINDOW_HEIGHT`).
pub const SCREEN_WIDTH: i32 = 765;
pub const SCREEN_HEIGHT: i32 = 503;

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

pub struct Ui {
    pub assets: Assets,
    interfaces: HashMap<u32, Layer>,
    /// Container id → the interface open inside it.
    subs: HashMap<u32, u32>,
    /// `BASE_WINDOW_ID`.
    root: Option<u32>,
    pub prefs: crate::client::preferences::Preferences,
    pub varc: HashMap<i32, i32>,
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
}

impl Ui {
    pub fn new(assets: Assets) -> Self {
        Ui {
            assets,
            interfaces: HashMap::new(),
            subs: HashMap::new(),
            root: None,
            prefs: Default::default(),
            varc: HashMap::new(),
            varc_strings: HashMap::new(),
            active: [None, None],
            started: std::time::Instant::now(),
            login_request: None,
            login_reply: -2,
            drag: None,
            mouse: (0, 0),
            prefs_file: None,
            safe_mode_at_start: false,
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
        self.layout_interface(interface, SCREEN_WIDTH, SCREEN_HEIGHT);
        self.run_on_load(interface);
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
}
