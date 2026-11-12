//! Sizes and positions of components. Port of `Class484.initSizes`, `Class246.method4204`,
//! `InteractableObject.method16099`, `Class480.method8044`, `Class442.method7403` and
//! `IdentikitDefinition.method913`.
//!
//! Left out on purpose: the `aBool7168` debug mode (it forces minimum sizes and clamps
//! positions) and resize hooks, which only run when the caller asks; every layout the
//! client runs here passes false.

use super::component::Comp;
use super::{CompRef, Layer, Ui, SCREEN_HEIGHT, SCREEN_WIDTH};
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

/// `method16099`: lays out every component of `layer` whose parent is `parent`. Containers
/// found on the way are laid out inside; the containers whose sub-interfaces still need it
/// are added to `subs`.
fn layout_level(layer: &mut Layer, parent: Option<u32>, w: i32, h: i32, subs: &mut Vec<(u32, i32, i32)>) {
    for i in 0..layer.len() {
        let Some(c) = layer[i].as_mut() else { continue };
        if c.parent != parent {
            continue;
        }
        size(c, w, h);
        position(c, w, h);
        clamp_scroll(c);
        // A created container would make the reference recurse into its own siblings
        // forever; only cache containers are followed.
        if c.kind == Kind::Container && c.slot == -1 {
            layout_container(layer, i, subs);
        }
    }
}

/// `method8044` for component `i` of `layer`: its children, its created children, then
/// (through `subs`) whatever interface is open in it.
fn layout_container(layer: &mut Layer, i: usize, subs: &mut Vec<(u32, i32, i32)>) {
    let Some(c) = layer[i].as_ref() else { return };
    let (id, (w, h)) = (c.id, inner_size(c));
    layout_level(layer, Some(id), w, h, subs);
    let Some(c) = layer[i].as_mut() else { return };
    let mut created = std::mem::take(&mut c.children);
    layout_level(&mut created, Some(id), w, h, subs);
    if let Some(c) = layer[i].as_mut() {
        c.children = created;
    }
    subs.push((id, w, h));
}

impl Ui {
    fn layout_subs(&mut self, subs: Vec<(u32, i32, i32)>) {
        for (container, w, h) in subs {
            if let Some(&interface) = self.subs.get(&container) {
                self.layout_interface(interface, w, h);
            }
        }
    }

    /// `method7403`: lays out a whole interface in a w × h area.
    pub fn layout_interface(&mut self, interface: u32, w: i32, h: i32) {
        if !self.load(interface) {
            return;
        }
        let mut subs = Vec::new();
        if let Some(layer) = self.interfaces.get_mut(&interface) {
            layout_level(layer, None, w, h, &mut subs);
        }
        self.layout_subs(subs);
    }

    /// `getParentLayer`'s size: the parent component, the container an interface is open
    /// in, or the screen.
    fn parent_size(&mut self, r: CompRef) -> (i32, i32) {
        let Some(c) = self.comp(r) else { return (SCREEN_WIDTH, SCREEN_HEIGHT) };
        let parent = c.parent;
        let container = match parent {
            Some(p) => Some(p),
            None => self.subs.iter().find(|(_, &i)| i == r.id >> 16).map(|(&p, _)| p),
        };
        container.and_then(|p| self.comp(CompRef::of(p))).map_or((SCREEN_WIDTH, SCREEN_HEIGHT), |p| (p.width, p.height))
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
        let mut subs = Vec::new();
        if let Some(layer) = self.interfaces.get_mut(&interface) {
            layout_level(layer, Some(id), w, h, &mut subs);
            if let Some(Some(c)) = layer.get_mut((id & 0xffff) as usize) {
                let mut created = std::mem::take(&mut c.children);
                layout_level(&mut created, Some(id), w, h, &mut subs);
                if let Some(Some(c)) = layer.get_mut((id & 0xffff) as usize) {
                    c.children = created;
                }
            }
        }
        subs.push((id, w, h));
        self.layout_subs(subs);
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
