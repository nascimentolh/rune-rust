//! Mouse and keyboard on interfaces, once per 20 ms tick. Port of
//! `ServerEnvironment.pulseSubInterfaces` → `client.method11768` (the hooks of the
//! components the mouse and keys reach), then the hook queues the tick runs in order:
//! `aClass482_7233`, `aClass482_7404`, `PENDING_HOOK_REQUESTS` (`client.java:2788-2890`).
//!
//! Dragging is in `drag.rs`; here the pass starts a drag, notes where the dragged component and
//! its bounds are, and holds back press hooks while one is under way.
//!
//! Left out on purpose: key bindings (`aBool1424`), the sprite click mask (no login sprite uses
//! one), `noClickThrough`, and the hooks that wait for changed vars, inventories, stats or clan
//! settings — none exists before login.

use super::component::Comp;
use super::drag::{draggable, Bounds, Drag};
use super::{CompRef, Layer, Ui, VarpChanges};
use crate::cache::interface::Kind;
use crate::client::cs2::{run_hook, Hook};
use crate::client::raster::Clip;
use std::collections::HashMap;

/// What the mouse and keyboard did since the last tick.
#[derive(Clone, Debug, Default)]
pub struct Input {
    pub mouse: (i32, i32),
    /// `leftButtonDown`.
    pub button_down: bool,
    /// `rightButtonDown`, `middleButtonDown`.
    pub other_buttons_down: [bool; 2],
    /// Where the left button went down (`mouseRecords` head, click type 0).
    pub click: Option<(i32, i32)>,
    /// `KEYS_PRESSED`: (code, char) — presses carry char 65535, typed characters code -1.
    pub keys: Vec<(i32, i32)>,
    /// `anInt7191`: wheel notches this tick, positive towards the user (`getWheelRotation`).
    pub wheel: i32,
}

/// Content types whose input the 3D toolkit handles; their components are skipped (`continue`).
const TOOLKIT_INPUT: [u16; 6] = [1337, 1403, 1338, 1400, 1401, 1406];

#[derive(Default)]
struct Queues {
    /// `aClass482_7233`: every-tick hooks.
    early: Vec<Hook>,
    /// `aClass482_7404`: button-up and mouse-leave hooks.
    release: Vec<Hook>,
    /// `PENDING_HOOK_REQUESTS`.
    pending: Vec<Hook>,
}

struct Pass<'a> {
    interfaces: &'a mut HashMap<u32, Layer>,
    subs: &'a HashMap<u32, u32>,
    input: &'a Input,
    /// The position tested against components: the click's, else the cursor's.
    at: (i32, i32),
    queues: Queues,
    /// `aClass118_7257` and its state; a drag starts during the pass.
    drag: Option<Drag>,
    /// The player variable changes so far (`anInt7453`, `anIntArray7379`).
    varp_changes: &'a VarpChanges,
    /// The menu's top entry so far: a component and its option (see `menu.rs`).
    menu_top: Option<(CompRef, i32)>,
}

fn inside((px, py): (i32, i32), b: Clip) -> bool {
    px >= b.left && py >= b.top && px < b.right && py < b.bottom
}

fn hook(c: &Comp, field: &str, mouse: (i32, i32), key: (i32, i32)) -> Option<Hook> {
    let args = c.hook(field)?.clone();
    Some(Hook { source: Some(CompRef { id: c.id, slot: c.slot }), args, mouse, key, op: 0 })
}

impl Pass<'_> {
    fn layer(&mut self, layer: &mut Layer, parent: Option<u32>, clip: Clip, ox: i32, oy: i32) {
        for i in 0..layer.len() {
            let Some(c) = layer[i].as_mut() else { continue };
            if c.parent != parent {
                continue;
            }
            let (x, y) = (c.x + ox, c.y + oy);
            let bounds = if c.kind == Kind::Other(2) {
                clip
            } else {
                let extra = i32::from(c.kind == Kind::Line);
                Clip {
                    left: x.max(clip.left),
                    top: y.max(clip.top),
                    right: (x + c.width + extra).min(clip.right),
                    bottom: (y + c.height + extra).min(clip.bottom),
                }
            };
            let me = CompRef { id: c.id, slot: c.slot };
            let drag_bounds = self.drag.as_ref().and_then(|d| d.bounds);
            let takes_input = c.kind == Kind::Container
                || c.uses_scripts
                || c.settings != 0
                || drag_bounds == Some(Bounds::Comp(me))
                || [1338, 1406, 1337, 1403].contains(&c.content_type);
            if !takes_input || c.hidden {
                continue;
            }
            if let Some(d) = self.drag.as_mut().filter(|d| d.comp == me && draggable(c)) {
                d.seen_at = Some((x, y));
            }
            if bounds.left >= bounds.right || bounds.top >= bounds.bottom || TOOLKIT_INPUT.contains(&c.content_type) {
                continue;
            }
            // `iComponentOnIComponent`: its options join the menu while the mouse is over it.
            if inside(self.input.mouse, bounds) {
                if let Some(op) = super::menu::top_option(c) {
                    self.menu_top = Some((me, op));
                }
            }
            // `Node_Sub14.method12221` on a click inside it.
            if let Some((cx, cy)) = self.input.click.filter(|&p| inside(p, bounds)) {
                if self.drag.is_none() && draggable(c) {
                    self.drag = Some(Drag::new(me, (cx - x, cy - y)));
                }
            }
            if let Some(d) = self.drag.as_mut().filter(|d| d.bounds == Some(Bounds::Comp(me))) {
                d.area = Some((x, y, c.width, c.height));
            }
            if c.uses_scripts || c.content_type != 0 {
                self.hooks(c, x, y, bounds);
            }
            if c.kind == Kind::Container {
                let (id, sx, sy) = (c.id, x - c.scroll_x, y - c.scroll_y);
                let mut created = std::mem::take(&mut c.children);
                self.layer(layer, Some(id), bounds, sx, sy);
                self.layer(&mut created, Some(id), bounds, sx, sy);
                if let Some(c) = layer[i].as_mut() {
                    c.children = created;
                }
                if let Some(&interface) = self.subs.get(&id) {
                    if let Some(mut sub) = self.interfaces.remove(&interface) {
                        self.layer(&mut sub, None, bounds, x, y);
                        self.interfaces.insert(interface, sub);
                    }
                }
            }
        }
    }

    /// The hook part of `method11768` for one component at (x, y).
    fn hooks(&mut self, c: &mut Comp, x: i32, y: i32, bounds: Clip) {
        let input = self.input;
        let over = inside(self.at, bounds);
        let q = &mut self.queues;
        if over && input.wheel != 0 {
            q.pending.extend(hook(c, "anObjectArray1412", (0, input.wheel), (0, 0)));
        }
        // While something is dragged, no press or hold reaches anything.
        let dragging = self.drag.is_some();
        let held = input.button_down && over && !dragging;
        let clicked = input.click.filter(|&p| inside(p, bounds) && !dragging);
        let cursor = (input.mouse.0 - x, input.mouse.1 - y);
        if let Some((cx, cy)) = clicked.filter(|_| !c.pressed) {
            c.pressed = true;
            q.pending.extend(hook(c, "anObjectArray1386", (cx - x, cy - y), (0, 0)));
        }
        if c.pressed && held {
            q.pending.extend(hook(c, "anObjectArray1319", cursor, (0, 0)));
        }
        if c.pressed && !held {
            c.pressed = false;
            q.release.extend(hook(c, "anObjectArray1302", cursor, (0, 0)));
        }
        if held {
            q.pending.extend(hook(c, "anObjectArray1389", cursor, (0, 0)));
        }
        if !c.hovered && over {
            c.hovered = true;
            q.pending.extend(hook(c, "onMouseHoverScript", cursor, (0, 0)));
        }
        if c.hovered && over {
            q.pending.extend(hook(c, "popupScript", cursor, (0, 0)));
        }
        if c.hovered && !over {
            c.hovered = false;
            q.release.extend(hook(c, "onMouseLeaveScript", cursor, (0, 0)));
        }
        q.early.extend(hook(c, "anObjectArray1409", (0, 0), (0, 0)));
        for &key in &input.keys {
            q.pending.extend(hook(c, "anObjectArray1292", (0, 0), key));
        }
        // `client.java:1520-1543`: a hook waiting on player variables runs when one of its
        // variables changed since it last looked (any change if it lists none, or if more than
        // 32 changes went by).
        let changes = self.varp_changes;
        if c.hook("anObjectArray1397").is_some() && changes.count > c.varp_seen {
            let fire = match &c.varp_triggers {
                Some(ids) if changes.count - c.varp_seen <= 32 => {
                    (c.varp_seen..changes.count).any(|i| ids.contains(&changes.recent[(i & 31) as usize]))
                }
                _ => true,
            };
            if fire {
                q.pending.extend(hook(c, "anObjectArray1397", (0, 0), (0, 0)));
            }
            c.varp_seen = changes.count;
        }
    }
}

impl Ui {
    /// One client tick of interface input, then its hooks.
    pub fn tick(&mut self, input: &Input) {
        self.tick_music();
        let Some(root) = self.root else { return };
        self.mouse = input.mouse;
        self.mouse_buttons = [input.button_down, input.other_buttons_down[0], input.other_buttons_down[1]];
        self.varp_changes.record();
        self.start_drag_pass();
        let Some(mut layer) = self.interfaces.remove(&root) else { return };
        let full = Clip { left: 0, top: 0, right: self.width, bottom: self.height };
        let mut pass = Pass {
            interfaces: &mut self.interfaces,
            subs: &self.subs,
            input,
            at: input.click.unwrap_or(input.mouse),
            queues: Queues::default(),
            drag: self.drag.take(),
            varp_changes: &self.varp_changes,
            menu_top: None,
        };
        pass.layer(&mut layer, None, full, 0, 0);
        let Pass { queues, drag, menu_top, .. } = pass;
        self.drag = drag;
        self.interfaces.insert(root, layer);
        self.settle_drag();
        // Resize hooks were appended to `PENDING_HOOK_REQUESTS` before this tick's.
        let resized = std::mem::take(&mut self.resize_hooks);
        for h in queues.early.into_iter().chain(queues.release).chain(resized).chain(queues.pending) {
            // A created component deleted since its hook was queued doesn't run it.
            if h.source.is_some_and(|s| s.slot >= 0 && self.comp(s).is_none()) {
                continue;
            }
            run_hook(self, &h, 200_000);
        }
        if self.drag.is_some() {
            self.drag_tick(input.mouse, input.button_down);
        }
        // A left click runs the menu's top entry (not while dragging).
        if let (Some(_), Some((r, op)), None) = (input.click, menu_top, &self.drag) {
            self.click_option(r, op);
        }
    }
}

impl Ui {
    /// One scripted input step, one tick each (for the tools and the self-test): `click x y`
    /// (press, then release on the next tick), `type text` (typed characters, code -1) or
    /// `key code` (a client key code press, char 65535); `reply code` sets the login reply
    /// code the window's scripts read, then runs a tick; `drag x y x2 y2` presses at (x, y),
    /// moves to (x2, y2) over a few ticks and lets go; `wheel n` turns the wheel n notches;
    /// `worlds n` receives a world list of n worlds.
    pub fn play(&mut self, step: &str) -> Result<(), String> {
        let (verb, rest) = step.split_once(' ').unwrap_or((step, ""));
        let num = |s: &str| s.parse::<i32>().map_err(|_| format!("bad number in {step:?}"));
        match verb {
            "click" => {
                let (x, y) = rest.split_once(' ').ok_or(format!("click needs x y: {step:?}"))?;
                let at = (num(x)?, num(y)?);
                self.tick(&Input { mouse: at, button_down: true, click: Some(at), ..Default::default() });
                self.tick(&Input { mouse: at, ..Default::default() });
            }
            "drag" => {
                let n: Vec<i32> = rest.split(' ').map(num).collect::<Result<_, _>>()?;
                let [x, y, x2, y2] = n[..] else { return Err(format!("drag needs x y x2 y2: {step:?}")) };
                self.tick(&Input { mouse: (x, y), button_down: true, click: Some((x, y)), ..Default::default() });
                for i in 1..=4 {
                    let at = (x + (x2 - x) * i / 4, y + (y2 - y) * i / 4);
                    self.tick(&Input { mouse: at, button_down: true, ..Default::default() });
                }
                self.tick(&Input { mouse: (x2, y2), ..Default::default() });
            }
            // For the tools: a player variable as the server's `VARP_*` packet sets it.
            "varp" => {
                let (id, value) = rest.split_once(' ').ok_or(format!("varp needs id value: {step:?}"))?;
                self.set_varp(num(id)?, num(value)?);
                self.tick(&Input::default());
            }
            // For the tools: a lobby login went through as `name` (a member, nothing else
            // set), open the lobby window (with its top bar after `lobby-bar`).
            "lobby" | "lobby-bar" => {
                self.login_reply = 2;
                let reply = crate::protocol::login::encode_lobby_ok(rest, 0, 0);
                self.open_lobby(&crate::protocol::login::decode_lobby_ok(&reply[2..])?, verb == "lobby-bar")?;
                self.tick(&Input::default());
            }
            // For the tools: a `WORLD_LIST` with worlds 1..=n arrived (n players each, members
            // on odd numbers).
            "worlds" => {
                use crate::protocol::world_list::{encode, Location, WorldEntry, WorldList, MEMBERS};
                let worlds = (1..=num(rest)?)
                    .map(|number| WorldEntry {
                        number,
                        location: 0,
                        flags: if number % 2 == 1 { MEMBERS } else { 0 },
                        activity: "-".into(),
                        host: "127.0.0.1".into(),
                        players: number * 10,
                    })
                    .collect();
                let list = WorldList { locations: vec![Location { country: 0, name: "Local".into() }], worlds, checksum: 1 };
                self.worlds.receive(&encode(&list, true), std::time::Instant::now())?;
                self.tick(&Input::default());
            }
            // For the tools: the game area becomes w × h, as after the window was resized in
            // mode 2 (`Class516.method8867(true)`).
            "size" => {
                let (w, h) = rest.split_once(' ').ok_or(format!("size needs w h: {step:?}"))?;
                (self.width, self.height) = (num(w)?, num(h)?);
                self.layout_after_resize();
                self.tick(&Input::default());
            }
            // For the tools: the score timing toolkit 1 would give, for Auto Setup.
            "opengl" => self.opengl_score = Some(num(rest)?),
            "wheel" => {
                let at = self.mouse;
                self.tick(&Input { mouse: at, wheel: num(rest)?, ..Default::default() });
            }
            "type" => self.tick(&Input { keys: rest.chars().map(|c| (-1, c as i32)).collect(), ..Default::default() }),
            "key" => self.tick(&Input { keys: vec![(num(rest)?, 65535)], ..Default::default() }),
            "reply" => {
                self.login_reply = num(rest)?;
                self.tick(&Input::default());
            }
            _ => return Err(format!("unknown input step {step:?}")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_hover_and_release_queue_their_hooks() {
        let mut c = Comp::created(5, Kind::Sprite, -1);
        c.width = 10;
        c.height = 10;
        for field in ["anObjectArray1386", "anObjectArray1302", "onMouseHoverScript", "onMouseLeaveScript"] {
            c.set_hook(field, Some(vec![super::super::component::Value::Int(1)]));
        }
        let mut layer: Layer = vec![Some(c)];
        let mut interfaces = HashMap::new();
        let subs = HashMap::new();
        let full = Clip { left: 0, top: 0, right: 100, bottom: 100 };
        let press = Input { mouse: (3, 4), button_down: true, click: Some((3, 4)), ..Default::default() };
        let mut pass = Pass {
            interfaces: &mut interfaces,
            subs: &subs,
            input: &press,
            at: (3, 4),
            queues: Queues::default(),
            drag: None,
            varp_changes: &VarpChanges::default(),
            menu_top: None,
        };
        pass.layer(&mut layer, Some(5), full, 0, 0);
        assert_eq!(pass.queues.pending.len(), 2); // press, then hover
        assert_eq!(pass.queues.pending[0].mouse, (3, 4));
        let away = Input { mouse: (50, 50), ..Default::default() };
        let mut pass = Pass {
            interfaces: &mut interfaces,
            subs: &subs,
            input: &away,
            at: (50, 50),
            queues: Queues::default(),
            drag: None,
            varp_changes: &VarpChanges::default(),
            menu_top: None,
        };
        pass.layer(&mut layer, Some(5), full, 0, 0);
        assert_eq!(pass.queues.release.len(), 2); // button up, then leave
    }
}
