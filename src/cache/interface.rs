//! Interface components (index 3: one archive per interface, one file per component).
//! Port of `IComponentDefinitions.readValues`, `decodeScript`, `decodeScript3` and
//! `SkyboxIndexLoader.getInterface`.

use super::buffer::Reader;
use super::enums::EnumValue;
use super::error::{corrupt, Result};

/// `ComponentType` (`jagex/ComponentType.java`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Container,
    Figure,
    Text,
    Sprite,
    Model,
    Line,
    /// Types 1, 2, 7 and 8: no extra fields.
    Other(u8),
}

impl Kind {
    pub fn from_id(id: u8) -> Self {
        match id {
            0 => Kind::Container,
            3 => Kind::Figure,
            4 => Kind::Text,
            5 => Kind::Sprite,
            6 => Kind::Model,
            9 => Kind::Line,
            other => Kind::Other(other),
        }
    }
}

/// A script hook's arguments: script id first, then its arguments (`decodeScript`).
#[derive(Clone, Debug, PartialEq)]
pub enum HookArg {
    Int(i32),
    Text(String),
}

pub type Hook = Option<Vec<HookArg>>;

/// What only some component kinds carry.
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    None,
    Container {
        scroll_width: u16,
        scroll_height: u16,
    },
    Sprite {
        sprite: i32,
        angle: u16,
        tiling: bool,
        alpha: bool,
        transparency: u8,
        border: u8,
        shadow: i32,
        v_flip: bool,
        h_flip: bool,
        colour: i32,
    },
    Model {
        model: i32,
        animation: i32,
    },
    Text {
        font: i32,
        monospaced: bool,
        text: String,
        line_height: u8,
        h_align: u8,
        v_align: u8,
        shadow: bool,
        colour: i32,
        transparency: u8,
        max_lines: u8,
    },
    Figure {
        colour: i32,
        filled: bool,
        transparency: u8,
    },
    Line {
        width: u8,
        colour: i32,
        direction: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    /// (interface << 16) | index.
    pub id: u32,
    pub version: i32,
    pub name: Option<String>,
    pub kind: Kind,
    pub content_type: u16,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    /// How width, height, x and y follow the parent (`aspect*Type`).
    pub width_mode: i8,
    pub height_mode: i8,
    pub x_mode: i8,
    pub y_mode: i8,
    /// Full id of the parent component, or `None` at the root.
    pub parent: Option<u32>,
    pub hidden: bool,
    pub content: Content,
    pub options: Vec<String>,
    /// `optionMask`: the settings `IFTargetParams` reads (drag depth `>> 18 & 7`, whole
    /// screen `>> 23 & 1`, ...).
    pub settings: u32,
    /// `aClass465_1365`: params declared on the component.
    pub params: std::collections::HashMap<i32, EnumValue>,
    /// `anInt1380`, `anInt1381`, `anInt1382`: how far (pixels) and how long (ticks) the mouse
    /// must move before a drag counts, and how the dragged component is drawn.
    pub drag_dead_zone: u8,
    pub drag_dead_time: u8,
    pub drag_render: u8,
    /// `varps`: the player variables the `anObjectArray1397` hook waits on.
    pub varps: Option<Vec<i32>>,
    pub on_load: Hook,
    pub on_mouse_over: Hook,
    pub on_mouse_leave: Hook,
    /// Every other hook, in file order (`anObjectArray1396` … `anObjectArray1405`).
    pub other_hooks: Vec<Hook>,
}

fn hook(r: &mut Reader) -> Result<Hook> {
    let n = r.u8()?;
    if n == 0 {
        return Ok(None);
    }
    (0..n)
        .map(|_| match r.u8()? {
            0 => Ok(HookArg::Int(r.i32()?)),
            1 => Ok(HookArg::Text(r.string()?)),
            other => corrupt(format!("unknown hook argument type {other}")),
        })
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

/// `decodeScript3`: a count, then that many ints (`None` for 0).
fn int_list(r: &mut Reader) -> Result<Option<Vec<i32>>> {
    let n = r.u8()?;
    if n == 0 {
        return Ok(None);
    }
    (0..n).map(|_| r.i32()).collect::<Result<Vec<_>>>().map(Some)
}

/// The client's own text tweak in `readValues`: the game's name becomes "Open 727".
fn rename(text: String) -> String {
    if text.to_lowercase().contains("runescape") {
        text.replace("runescape", "Open 727").replace("RuneScape", "Open 727").replace("Runescape", "Open 727")
    } else {
        text
    }
}

impl Component {
    pub fn decode(id: u32, data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let version = match r.u8()? {
            255 => -1,
            v => v as i32,
        };
        let mut type_id = r.u8()?;
        let name = if type_id & 0x80 != 0 {
            type_id &= 0x7f;
            Some(r.string()?)
        } else {
            None
        };
        let kind = Kind::from_id(type_id);
        let content_type = r.u16()?;
        let (x, y, width, height) = (r.i16()?, r.i16()?, r.u16()?, r.u16()?);
        let (width_mode, height_mode, x_mode, y_mode) = (r.i8()?, r.i8()?, r.i8()?, r.i8()?);
        let parent = match r.u16()? {
            0xffff => None,
            p => Some(p as u32 + (id & 0xffff_0000)),
        };
        let flags = r.u8()?;
        let hidden = flags & 1 != 0;

        let content = match kind {
            Kind::Container => {
                let (scroll_width, scroll_height) = (r.u16()?, r.u16()?);
                if version < 0 {
                    r.u8()?; // no click-through
                }
                Content::Container { scroll_width, scroll_height }
            }
            Kind::Sprite => {
                let sprite = r.i32()?;
                let angle = r.u16()?;
                let f = r.u8()?;
                let (transparency, border, shadow) = (r.u8()?, r.u8()?, r.i32()?);
                let (v_flip, h_flip, colour) = (r.u8()? == 1, r.u8()? == 1, r.i32()?);
                if version >= 3 {
                    r.u8()?; // click mask
                }
                Content::Sprite {
                    sprite,
                    angle,
                    tiling: f & 1 != 0,
                    alpha: f & 2 != 0,
                    transparency,
                    border,
                    shadow,
                    v_flip,
                    h_flip,
                    colour,
                }
            }
            Kind::Model => {
                let model = r.big_smart_or_none()?;
                let f = r.u8()?;
                if f & 1 == 1 {
                    r.bytes(12)?; // origin x/y, pitch, roll, yaw, scale
                } else if f & 2 == 2 {
                    r.bytes(14)?; // origin x/y/z, pitch, roll, yaw, scale
                }
                let animation = r.big_smart_or_none()?;
                if width_mode != 0 {
                    r.u16()?;
                }
                if height_mode != 0 {
                    r.u16()?;
                }
                Content::Model { model, animation }
            }
            Kind::Text => {
                let font = r.big_smart_or_none()?;
                let monospaced = version >= 2 && r.u8()? == 1;
                let text = rename(r.string()?);
                let (line_height, h_align, v_align) = (r.u8()?, r.u8()?, r.u8()?);
                let (shadow, colour, transparency) = (r.u8()? == 1, r.i32()?, r.u8()?);
                let max_lines = if version >= 0 { r.u8()? } else { 0 };
                Content::Text { font, monospaced, text, line_height, h_align, v_align, shadow, colour, transparency, max_lines }
            }
            Kind::Figure => Content::Figure { colour: r.i32()?, filled: r.u8()? == 1, transparency: r.u8()? },
            Kind::Line => Content::Line { width: r.u8()?, colour: r.i32()?, direction: r.u8()? == 1 },
            Kind::Other(_) => Content::None,
        };

        // Options and cursors.
        let settings = r.u24()?;
        let mut key = r.u8()?;
        while key != 0 {
            r.u8()?;
            r.bytes(2)?;
            key = r.u8()?;
        }
        r.string()?; // use-on name
        let counts = r.u8()?;
        let options = (0..counts & 0xf).map(|_| r.string()).collect::<Result<Vec<_>>>()?;
        let cursor_count = counts >> 4;
        if cursor_count > 0 {
            r.bytes(3)?;
        }
        if cursor_count > 1 {
            r.bytes(3)?;
        }
        r.string()?; // op name
        let (drag_dead_zone, drag_dead_time, drag_render) = (r.u8()?, r.u8()?, r.u8()?);
        r.string()?; // use option
        if settings >> 11 & 0x7f != 0 {
            r.bytes(6)?; // target cursors
        }
        // `aClass465_1365`: the component's params (key u24, then an int or a GJ string).
        let mut params = std::collections::HashMap::new();
        if version >= 0 {
            r.u16()?; // mouse-over cursor
            let ints = r.u8()?;
            for _ in 0..ints {
                let key = r.u24()? as i32;
                params.insert(key, EnumValue::Int(r.i32()?));
            }
            let strings = r.u8()?;
            for _ in 0..strings {
                let key = r.u24()? as i32;
                // `readGJString`: a 0 byte, then the text.
                r.u8()?;
                params.insert(key, EnumValue::Text(r.string()?));
            }
        }

        // Script hooks, in the order `readValues` reads them.
        let on_load = hook(&mut r)?;
        let on_mouse_over = hook(&mut r)?;
        let on_mouse_leave = hook(&mut r)?;
        let mut other_hooks = Vec::new();
        for _ in 0..7 {
            other_hooks.push(hook(&mut r)?);
        }
        if version >= 0 {
            other_hooks.push(hook(&mut r)?);
        }
        for _ in 0..10 {
            other_hooks.push(hook(&mut r)?);
        }
        // Trigger lists: `varps` (what `anObjectArray1397` waits on), then four the hooks for
        // inventories, stats and client variables use, read past until those are ported.
        let varps = int_list(&mut r)?;
        for _ in 0..4 {
            int_list(&mut r)?;
        }
        Ok(Component {
            id,
            version,
            name,
            kind,
            content_type,
            x,
            y,
            width,
            height,
            width_mode,
            height_mode,
            x_mode,
            y_mode,
            parent,
            hidden,
            content,
            options,
            settings,
            params,
            drag_dead_zone,
            drag_dead_time,
            drag_render,
            varps,
            on_load,
            on_mouse_over,
            on_mouse_leave,
            other_hooks,
        })
    }
}

/// Every component of one interface (files of its archive), in file order.
pub fn decode_interface(interface: u32, files: &[Vec<u8>]) -> Result<Vec<Component>> {
    files.iter().enumerate().map(|(i, data)| Component::decode(interface << 16 | i as u32, data)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_the_game() {
        assert_eq!(rename("Welcome to RuneScape".into()), "Welcome to Open 727");
        assert_eq!(rename("Hello".into()), "Hello");
    }
}
