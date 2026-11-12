//! Dragging a component. Port of `Node_Sub14.method12221` (start), the drag parts of
//! `client.method11768` (see `input.rs`) and `EntitySpotAnim.method2827` (every tick, after
//! the hook queues). The login screen's volume sliders drag their knob inside the bar
//! (`instr6214`) and set the volume from the release hook (`anObjectArray1394`).
//!
//! Not ported: the drop target (`aClass118_7370`, tells the server where an item went) and the
//! menu click a drag that never moved turns into (`QuestIndexLoader.method4164`, needs the
//! right-click menu).

use super::component::Comp;
use super::{CompRef, Ui, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::client::cs2::{run_hook, Hook};

/// Where a dragged component may go (`aClass118_7247`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bounds {
    /// `InputSubscriberType.aClass118_2763`: the whole window.
    Screen,
    Comp(CompRef),
}

#[derive(Clone, Debug)]
pub struct Drag {
    /// `aClass118_7257`.
    pub comp: CompRef,
    /// `aClass118_7247`; `None` until the tick that started the drag has finished its pass
    /// (the reference finds it at once, but nothing reads it before then).
    pub bounds: Option<Bounds>,
    /// `anInt7361`, `anInt7362`: where in the component the mouse grabbed it.
    pub grab: (i32, i32),
    /// `SendFinishedCutsceneAction.anInt8308`: ticks since the start.
    pub ticks: i32,
    /// `aBool7364`: moved past the dead zone after the dead time.
    pub moved: bool,
    /// `aBool7369` with `anInt7252`, `anInt7215`: where this tick's pass found the component.
    pub seen_at: Option<(i32, i32)>,
    /// `aBool7403` with `anInt7432`, `anInt7265`, `anInt7367`, `anInt7476`: the bounds' box.
    pub area: Option<(i32, i32, i32, i32)>,
}

/// `IFTargetParams.depthFlags`.
fn depth(settings: u32) -> u32 {
    settings >> 18 & 7
}

/// `IFTargetParams.bit23Enabled`.
pub fn whole_screen(settings: u32) -> bool {
    settings >> 23 & 1 != 0
}

/// `ApplyHitmarkCutsceneAction.method14643`: the component can be dragged.
pub fn draggable(c: &Comp) -> bool {
    depth(c.settings) > 0 || whole_screen(c.settings) || c.drag_parent.is_some()
}

impl Drag {
    /// `method12221`'s fields for a drag of `comp` grabbed at `grab`.
    pub fn new(comp: CompRef, grab: (i32, i32)) -> Self {
        Drag { comp, bounds: None, grab, ticks: 0, moved: false, seen_at: None, area: None }
    }

    /// Where the dragged component goes for the mouse at `mouse` (`method2827`, the same
    /// clamping as `IComponentDefinitions.render`): inside the area, left/top edge first.
    pub fn position(&self, mouse: (i32, i32), size: (i32, i32)) -> Option<(i32, i32)> {
        let (ax, ay, aw, ah) = self.area?;
        let mut x = (mouse.0 - self.grab.0).max(ax);
        if x + size.0 > aw + ax {
            x = aw + ax - size.0;
        }
        let mut y = (mouse.1 - self.grab.1).max(ay);
        if y + size.1 > ay + ah {
            y = ah + ay - size.1;
        }
        Some((x, y))
    }
}

impl Ui {
    /// `IComponentDefinitions.getParentLayer`: the parent component, or at an interface's
    /// root the container it is open in.
    fn parent_of(&mut self, r: CompRef) -> Option<CompRef> {
        match self.comp(r)?.parent {
            Some(p) => Some(CompRef::of(p)),
            None => self.subs.iter().find(|(_, &i)| i == r.id >> 16).map(|(&container, _)| CompRef::of(container)),
        }
    }

    /// `TexturesPreference.method12880` (with `client.method11634`): what a drag of `r` stays in.
    fn bounds_of(&mut self, r: CompRef) -> Option<Bounds> {
        let c = self.comp(r)?;
        let (settings, link) = (c.settings, c.drag_parent);
        if whole_screen(settings) {
            return Some(Bounds::Screen);
        }
        if depth(settings) == 0 {
            return link.map(Bounds::Comp);
        }
        let mut at = r;
        for _ in 0..depth(settings) {
            match self.parent_of(at) {
                Some(p) => at = p,
                None => return Some(Bounds::Screen),
            }
        }
        Some(Bounds::Comp(at))
    }

    /// `method12221`, from a script (`IF_DRAGPICKUP`, `CC_DRAGPICKUP`).
    pub fn pick_up(&mut self, r: CompRef, grab: (i32, i32)) {
        if self.drag.is_none() && self.comp(r).is_some_and(|c| draggable(c)) {
            self.drag = Some(Drag::new(r, grab));
            self.settle_drag();
        }
    }

    /// Finds the bounds of a drag that just started.
    pub(super) fn settle_drag(&mut self) {
        let Some(comp) = self.drag.as_ref().filter(|d| d.bounds.is_none()).map(|d| d.comp) else { return };
        match self.bounds_of(comp) {
            Some(b) => self.drag.as_mut().into_iter().for_each(|d| d.bounds = Some(b)),
            None => self.drag = None,
        }
    }

    /// Clears what the last pass saw (`MaterialProp23`: `aBool7369`, `aBool7403`), with the
    /// whole window as the area of a whole-screen drag (`ServerEnvironment.pulseSubInterfaces`).
    pub(super) fn start_drag_pass(&mut self) {
        if let Some(d) = &mut self.drag {
            d.seen_at = None;
            d.area = (d.bounds == Some(Bounds::Screen)).then_some((0, 0, SCREEN_WIDTH, SCREEN_HEIGHT));
        }
    }

    /// `EntitySpotAnim.method2827`: move, run the drag hooks, and drop on release.
    pub(super) fn drag_tick(&mut self, mouse: (i32, i32), button_down: bool) {
        let Some(mut d) = self.drag.take() else { return };
        d.ticks += 1;
        let Some(c) = self.comp(d.comp) else { return };
        let (size, dead_zone, dead_time) = ((c.width, c.height), c.drag_dead_zone, c.drag_dead_time);
        let (Some(seen), Some(pos)) = (d.seen_at, d.position(mouse, size)) else {
            if d.ticks <= 1 {
                self.drag = Some(d);
            }
            return;
        };
        let (ax, ay, ..) = d.area.unwrap_or_default();
        let rel = match d.bounds {
            Some(Bounds::Comp(b)) => {
                let (sx, sy) = self.comp(b).map_or((0, 0), |b| (b.scroll_x, b.scroll_y));
                (pos.0 - ax + sx, pos.1 - ay + sy)
            }
            _ => pos,
        };
        // `mouseRecorder.method3568`: a button is down (only the left one is read here).
        if !button_down {
            if d.moved {
                self.run_drag_hook(d.comp, "anObjectArray1394", rel);
            }
            return;
        }
        if d.ticks > dead_time {
            let (dx, dy) = (pos.0 - seen.0, pos.1 - seen.1);
            if dx > dead_zone || dx < -dead_zone || dy > dead_zone || dy < -dead_zone {
                d.moved = true;
            }
        }
        let (comp, moved) = (d.comp, d.moved);
        self.drag = Some(d);
        if moved {
            self.run_drag_hook(comp, "anObjectArray1451", rel);
        }
    }

    /// `CS2Executor.executeHookInner`: runs at once, not queued.
    fn run_drag_hook(&mut self, r: CompRef, field: &str, mouse: (i32, i32)) {
        let Some(args) = self.comp(r).and_then(|c| c.hook(field).cloned()) else { return };
        run_hook(self, &Hook { source: Some(r), args, mouse, key: (0, 0) }, 200_000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_stays_inside_the_area() {
        let mut d = Drag::new(CompRef::of(1), (4, 4));
        d.area = Some((100, 50, 112, 16));
        assert_eq!(d.position((50, 0), (16, 16)), Some((100, 50)));
        assert_eq!(d.position((300, 300), (16, 16)), Some((196, 50)));
        assert_eq!(d.position((154, 58), (16, 16)), Some((150, 50)));
    }
}
