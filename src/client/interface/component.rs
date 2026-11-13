//! A component as the client holds it while running: the cache definition plus
//! everything scripts change (`IComponentDefinitions`' mutable fields).

use super::CompRef;
use crate::cache::interface::{Component as Def, Content, HookArg, Kind};

/// A hook argument once a script has set it (`popParamsDynamic`): int, string or long.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Text(String),
    Long(i64),
}

/// The cache's `onLoadScript` hook.
pub const ON_LOAD: &str = "onLoadScript";

#[derive(Clone, Debug)]
pub struct Comp {
    /// `idHash`: (interface << 16) | index; a created child shares its parent's.
    pub id: u32,
    /// `slotId`: -1 for a cache component, else its place among the parent's created children.
    pub slot: i32,
    /// Full id of the parent, `None` at an interface root.
    pub parent: Option<u32>,
    pub kind: Kind,
    pub content_type: u16,
    pub base_x: i32,
    pub base_y: i32,
    pub base_width: i32,
    pub base_height: i32,
    pub width_mode: i8,
    pub height_mode: i8,
    pub x_mode: i8,
    pub y_mode: i8,
    /// `aspectX` / `aspectY`, used by size mode 4 (never set from the cache: always 1).
    pub aspect_x: i32,
    pub aspect_y: i32,
    /// Laid-out box relative to the parent (`x`, `y`, `width`, `height`).
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub hidden: bool,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub scroll_width: i32,
    pub scroll_height: i32,
    pub sprite: i32,
    pub tiling: bool,
    pub transparency: i32,
    pub border: i32,
    pub sprite_shadow: i32,
    pub v_flip: bool,
    pub h_flip: bool,
    pub colour: i32,
    pub filled: bool,
    pub font: i32,
    pub text: String,
    /// `anInt1358`: line height for text (0 = the font's).
    pub line_height: i32,
    pub h_align: i32,
    pub v_align: i32,
    pub text_shadow: bool,
    pub max_lines: i32,
    pub line_width: i32,
    pub line_direction: bool,
    /// `slotChildren`: components created by `CC_CREATE`.
    pub children: Vec<Option<Comp>>,
    /// Script hooks by the reference field they fill, each: script id, then its arguments.
    pub hooks: Vec<(&'static str, Vec<Value>)>,
    /// `usesScripts`: some hook was decoded or set; only such components (and containers)
    /// take part in input (`client.method11768`).
    pub uses_scripts: bool,
    /// `aBool1286`: a click started on it and the button is still down.
    pub pressed: bool,
    /// `aBool1440`: the mouse is over it.
    pub hovered: bool,
    /// `optionNames`: the right-click options' texts (option n is `options[n - 1]`).
    pub options: Vec<String>,
    /// `targetParams.settingsHash` (the server can override it after login; not before).
    pub settings: u32,
    /// `aClass465_1365`: params, from the cache and `CC_SETPARAM_INT`.
    pub params: std::collections::HashMap<i32, crate::cache::enums::EnumValue>,
    /// `varps`: the player variables the `anObjectArray1397` hook waits on (`None` = any).
    pub varp_triggers: Option<Vec<i32>>,
    /// `anInt1443`: how many variable changes this component has seen.
    pub varp_seen: u32,
    /// `aClass118_1379`: the component a drag of this one stays inside (`instr6214`).
    pub drag_parent: Option<CompRef>,
    /// `anInt1380`, `anInt1381`, `anInt1382` (see `cache::interface::Component`).
    pub drag_dead_zone: i32,
    pub drag_dead_time: i32,
    pub drag_render: i32,
}

/// The cache's other hooks in `readValues` order; `anObjectArray1393` only from version 0.
const HOOKS_BEFORE_VERSIONED: [&str; 7] =
    ["anObjectArray1396", "anObjectArray1400", "anObjectArray1397", "mouseLeaveScript", "anObjectArray1387", "anObjectArray1409", "params"];
const HOOKS_AFTER_VERSIONED: [&str; 10] = [
    "popupScript",
    "anObjectArray1386",
    "anObjectArray1319",
    "anObjectArray1302",
    "anObjectArray1389",
    "anObjectArray1451",
    "anObjectArray1394",
    "anObjectArray1412",
    "anObjectArray1403",
    "anObjectArray1405",
];

fn hook_values(args: &[HookArg]) -> Vec<Value> {
    args.iter()
        .map(|a| match a {
            HookArg::Int(v) => Value::Int(*v),
            HookArg::Text(t) => Value::Text(t.clone()),
        })
        .collect()
}

impl Comp {
    /// A blank component made by `CC_CREATE` (`method2738`): fields at their Java defaults.
    pub fn created(parent_id: u32, kind: Kind, slot: i32) -> Self {
        Comp {
            id: parent_id,
            slot,
            parent: Some(parent_id),
            kind,
            content_type: 0,
            base_x: 0,
            base_y: 0,
            base_width: 0,
            base_height: 0,
            width_mode: 0,
            height_mode: 0,
            x_mode: 0,
            y_mode: 0,
            aspect_x: 1,
            aspect_y: 1,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            hidden: false,
            scroll_x: 0,
            scroll_y: 0,
            scroll_width: 0,
            scroll_height: 0,
            sprite: -1,
            tiling: false,
            transparency: 0,
            border: 0,
            sprite_shadow: 0,
            params: Default::default(),
            v_flip: false,
            h_flip: false,
            colour: 0,
            filled: false,
            font: -1,
            text: String::new(),
            line_height: 0,
            h_align: 0,
            v_align: 0,
            text_shadow: false,
            max_lines: 0,
            line_width: 1,
            line_direction: false,
            children: Vec::new(),
            hooks: Vec::new(),
            uses_scripts: false,
            pressed: false,
            hovered: false,
            options: Vec::new(),
            settings: 0,
            varp_triggers: None,
            varp_seen: 0,
            drag_parent: None,
            drag_dead_zone: 0,
            drag_dead_time: 0,
            drag_render: 0,
        }
    }

    /// The live component for a cache definition (`readValues` fills the same fields).
    pub fn from_def(d: &Def) -> Self {
        let mut c = Comp::created(0, d.kind, -1);
        c.id = d.id;
        c.parent = d.parent;
        c.content_type = d.content_type;
        (c.base_x, c.base_y) = (d.x as i32, d.y as i32);
        (c.base_width, c.base_height) = (d.width as i32, d.height as i32);
        (c.width_mode, c.height_mode, c.x_mode, c.y_mode) = (d.width_mode, d.height_mode, d.x_mode, d.y_mode);
        c.hidden = d.hidden;
        c.settings = d.settings;
        c.params = d.params.clone();
        c.options = d.options.clone();
        c.varp_triggers = d.varps.clone();
        (c.drag_dead_zone, c.drag_dead_time, c.drag_render) = (d.drag_dead_zone as i32, d.drag_dead_time as i32, d.drag_render as i32);
        match &d.content {
            Content::Container { scroll_width, scroll_height } => {
                (c.scroll_width, c.scroll_height) = (*scroll_width as i32, *scroll_height as i32)
            }
            Content::Sprite { sprite, tiling, transparency, border, shadow, v_flip, h_flip, colour, .. } => {
                c.sprite = *sprite;
                c.tiling = *tiling;
                c.transparency = *transparency as i32;
                c.border = *border as i32;
                c.sprite_shadow = *shadow;
                (c.v_flip, c.h_flip) = (*v_flip, *h_flip);
                c.colour = *colour;
            }
            Content::Text { font, text, line_height, h_align, v_align, shadow, colour, transparency, max_lines, .. } => {
                c.font = *font;
                c.text = text.clone();
                c.line_height = *line_height as i32;
                (c.h_align, c.v_align) = (*h_align as i32, *v_align as i32);
                c.text_shadow = *shadow;
                c.colour = *colour;
                c.transparency = *transparency as i32;
                c.max_lines = *max_lines as i32;
            }
            Content::Figure { colour, filled, transparency } => {
                c.colour = *colour;
                c.filled = *filled;
                c.transparency = *transparency as i32;
            }
            Content::Line { width, colour, direction } => {
                c.line_width = *width as i32;
                c.colour = *colour;
                c.line_direction = *direction;
            }
            Content::Model { .. } | Content::None => {}
        }
        let versioned: &[&'static str] = if d.version >= 0 { &["anObjectArray1393"] } else { &[] };
        let other = HOOKS_BEFORE_VERSIONED.iter().chain(versioned).chain(&HOOKS_AFTER_VERSIONED);
        let named = [(ON_LOAD, &d.on_load), ("onMouseHoverScript", &d.on_mouse_over), ("onMouseLeaveScript", &d.on_mouse_leave)]
            .into_iter()
            .chain(other.copied().zip(&d.other_hooks));
        for (field, hook) in named {
            if let Some(args) = hook {
                c.hooks.push((field, hook_values(args)));
                // `decodeScript` sets `usesScripts` for every hook it reads.
                c.uses_scripts = true;
            }
        }
        c
    }

    pub fn hook(&self, field: &str) -> Option<&Vec<Value>> {
        self.hooks.iter().find(|(f, _)| *f == field).map(|(_, v)| v)
    }

    /// Sets or clears (`None`) one hook; either way the component now `usesScripts`.
    pub fn set_hook(&mut self, field: &'static str, values: Option<Vec<Value>>) {
        self.uses_scripts = true;
        self.hooks.retain(|(f, _)| *f != field);
        if let Some(v) = values {
            self.hooks.push((field, v));
        }
    }
}
