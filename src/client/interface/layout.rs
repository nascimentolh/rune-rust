//! Sizes and positions of components. Port of `Class484.initSizes`, `Class246.method4204`,
//! `InteractableObject.method16099`, `Class480.method8044`, `Class442.method7403` and
//! `IdentikitDefinition.method913`.
//!
//! A layout after the window changed size (`Class516.method8867(true)`) also queues the resize
//! hook (`anObjectArray1271`) of every component whose size changed (`initSizes`).
//!
//! Left out on purpose: the `aBool7168` debug mode (it forces minimum sizes and clamps
//! positions).

use super::component::Comp;
use super::{CompRef, Layer, Ui};
use crate::cache::interface::Kind;

/// `initSizes`.
fn size(c: &mut Comp, w: i32, h: i32) {
    match c.width_mode {
        0 => c.width = c.base_width,
        1 => c.width = w - c.base_width,
        2 => c.width = (c.base_width * w) >> 14,
        _ => {}
    }
    match c.height_mode {
        0 => c.height = c.base_height,
        1 => c.height = h - c.base_height,
        2 => c.height = (h * c.base_height) >> 14,
        _ => {}
    }
    if c.width_mode == 4 {
        c.width = c.aspect_x * c.height / c.aspect_y;
    }
    if c.height_mode == 4 {
        c.height = c.width * c.aspect_y / c.aspect_x;
    }
}

/// `method4204`.
fn position(c: &mut Comp, w: i32, h: i32) {
    let (bx, by) = (c.base_x, c.base_y);
    c.x = match c.x_mode {
        0 => bx,
        1 => bx + (w - c.width) / 2,
        2 => w - c.width - bx,
        3 => (bx * w) >> 14,
        4 => (w - c.width) / 2 + ((bx * w) >> 14),
        _ => w - c.width - ((bx * w) >> 14),
    };
    c.y = match c.y_mode {
        0 => by,
        1 => (h - c.height) / 2 + by,
        2 => h - c.height - by,
        3 => (h * by) >> 14,
        4 => (h - c.height) / 2 + ((h * by) >> 14),
        _ => h - c.height - ((h * by) >> 14),
    };
}

fn clamp_scroll(c: &mut Comp) {
    c.scroll_x = c.scroll_x.min(c.scroll_width - c.width).max(0);
    c.scroll_y = c.scroll_y.min(c.scroll_height - c.height).max(0);
}

/// The area a container's children are laid out in: its scroll size, or its own size.
fn inner_size(c: &Comp) -> (i32, i32) {
    (if c.scroll_width != 0 { c.scroll_width } else { c.width }, if c.scroll_height != 0 { c.scroll_height } else { c.height })
}

/// What one layout gathers on its way.
#[derive(Default)]
struct Lay {
    /// The containers whose sub-interfaces still need laying out, with their inner size.
    subs: Vec<(u32, i32, i32)>,
    /// `bool_3` of `initSizes`: the window changed size, so resize hooks may run.
    resize: bool,
    /// Components with a resize hook whose size changed.
    resized: Vec<CompRef>,
}

/// `anObjectArray1271`: run when the window's new size changed the component's.
const RESIZE_HOOK: &str = "anObjectArray1271";

/// `method16099`: lays out every component of `layer` whose parent is `parent`. Containers
/// found on the way are laid out inside.
fn layout_level(layer: &mut Layer, parent: Option<u32>, w: i32, h: i32, lay: &mut Lay) {
    for i in 0..layer.len() {
        let Some(c) = layer[i].as_mut() else { continue };
        if c.parent != parent {
            continue;
        }
        let before = (c.width, c.height);
        size(c, w, h);
        if lay.resize && before != (c.width, c.height) && c.hook(RESIZE_HOOK).is_some() {
            lay.resized.push(CompRef { id: c.id, slot: c.slot });
        }
        position(c, w, h);
        clamp_scroll(c);
        // A created container would make the reference recurse into its own siblings
        // forever; only cache containers are followed.
        if c.kind == Kind::Container && c.slot == -1 {
            layout_container(layer, i, lay);
        }
    }
}

/// `method8044` for component `i` of `layer`: its children, its created children, then
/// (through `subs`) whatever interface is open in it.
fn layout_container(layer: &mut Layer, i: usize, lay: &mut Lay) {
    let Some(c) = layer[i].as_ref() else { return };
    let (id, (w, h)) = (c.id, inner_size(c));
    layout_level(layer, Some(id), w, h, lay);
    let Some(c) = layer[i].as_mut() else { return };
    let mut created = std::mem::take(&mut c.children);
    layout_level(&mut created, Some(id), w, h, lay);
    if let Some(c) = layer[i].as_mut() {
        c.children = created;
    }
    lay.subs.push((id, w, h));
}

impl Ui {
    /// Lays out the sub-interfaces a layout reached, then queues the resize hooks it found
    /// (like `PENDING_HOOK_REQUESTS.append`).
    fn finish(&mut self, mut lay: Lay) {
        for (container, w, h) in std::mem::take(&mut lay.subs) {
            if let Some(&interface) = self.subs.get(&container) {
                self.lay_out(interface, w, h, &mut lay);
            }
        }
        for r in lay.resized {
            if let Some(args) = self.comp(r).and_then(|c| c.hook(RESIZE_HOOK).cloned()) {
                self.resize_hooks.push(crate::client::cs2::Hook { source: Some(r), args, ..Default::default() });
            }
        }
    }

    /// One interface and, inside it, the interfaces open in its containers.
    fn lay_out(&mut self, interface: u32, w: i32, h: i32, lay: &mut Lay) {
        if !self.load(interface) {
            return;
        }
        let mut inner = Lay { resize: lay.resize, ..Default::default() };
        if let Some(layer) = self.interfaces.get_mut(&interface) {
            layout_level(layer, None, w, h, &mut inner);
        }
        lay.resized.append(&mut inner.resized);
        for (container, w, h) in inner.subs {
            if let Some(&sub) = self.subs.get(&container) {
                self.lay_out(sub, w, h, lay);
            }
        }
    }

    /// `method7403`: lays out a whole interface in a w × h area.
    pub fn layout_interface(&mut self, interface: u32, w: i32, h: i32) {
        let mut lay = Lay::default();
        self.lay_out(interface, w, h, &mut lay);
        self.finish(lay);
    }

    /// `Class516.method8867(true)`: the root window laid out again in the new window size,
    /// with resize hooks.
    pub fn layout_after_resize(&mut self) {
        let Some(root) = self.root else { return };
        let mut lay = Lay { resize: true, ..Default::default() };
        let (w, h) = (self.width, self.height);
        self.lay_out(root, w, h, &mut lay);
        self.finish(lay);
    }

    /// `getParentLayer`'s size: the parent component, the container an interface is open
    /// in, or the screen.
    fn parent_size(&mut self, r: CompRef) -> (i32, i32) {
        let screen = (self.width, self.height);
        let Some(c) = self.comp(r) else { return screen };
        let parent = c.parent;
        let container = match parent {
            Some(p) => Some(p),
            None => self.subs.iter().find(|(_, &i)| i == r.id >> 16).map(|(&p, _)| p),
        };
        container.and_then(|p| self.comp(CompRef::of(p))).map_or(screen, |p| (p.width, p.height))
    }

    /// `method913`: sizes and places one component again after a script changed it.
    pub fn relayout(&mut self, r: CompRef) {
        let (w, h) = self.parent_size(r);
        if let Some(c) = self.comp(r) {
            size(c, w, h);
            position(c, w, h);
        }
    }

    /// `HostNameIdentifier.method483` (`method8044` on one component of an interface).
    pub fn layout_contents(&mut self, r: CompRef) {
        let interface = r.id >> 16;
        let Some((id, (w, h))) = self.comp(r).map(|c| (c.id, inner_size(c))) else { return };
        let mut lay = Lay::default();
        if let Some(layer) = self.interfaces.get_mut(&interface) {
            layout_level(layer, Some(id), w, h, &mut lay);
            if let Some(Some(c)) = layer.get_mut((id & 0xffff) as usize) {
                let mut created = std::mem::take(&mut c.children);
                layout_level(&mut created, Some(id), w, h, &mut lay);
                if let Some(Some(c)) = layer.get_mut((id & 0xffff) as usize) {
                    c.children = created;
                }
            }
        }
        lay.subs.push((id, w, h));
        self.finish(lay);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_positions_follow_the_modes() {
        let mut c = Comp::created(0, Kind::Container, -1);
        (c.base_width, c.base_height, c.width_mode, c.height_mode) = (10, 1 << 13, 1, 2);
        size(&mut c, 100, 50);
        assert_eq!((c.width, c.height), (90, 25));
        (c.base_x, c.base_y, c.x_mode, c.y_mode) = (5, 3, 1, 2);
        position(&mut c, 100, 50);
        assert_eq!((c.x, c.y), (5 + 5, 50 - 25 - 3));
    }
}
