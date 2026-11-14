//! Component options and the left click. The reference builds a right-click menu each tick from
//! the components under the mouse (`Class455.iComponentOnIComponent`: options 10..6, then 5..1,
//! each added on top) and a left click runs the top entry. Only that top entry is kept here:
//! the menu itself (drawing it, right-click, cancel, the other entry kinds) isn't ported yet.
//!
//! Running an option (`CutsceneAction_Sub10.method14603`): the component's `params` hook runs at
//! once with the option number, then, if its settings allow that option, `IF_OPn` goes to the
//! server.
//!
//! The scene adds "Walk here" for the tile under the mouse (`iComponentOnGroundTile`, menu
//! action 23); running it (`CutsceneAction_Sub4`, click type 23) sends `WALK`.

use super::component::Comp;
use super::{CompRef, Ui};
use crate::client::cs2::{run_hook, Hook};
use crate::protocol::client_packets;

/// The menu's top entry, the one a left click runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    /// A component's option (1..=10).
    Option(CompRef, i32),
    /// "Walk here" (action 23) to a scene tile.
    Walk((i32, i32)),
}

/// `IFTargetParams.clickOptionEnabled(n)`: settings bit n + 1.
fn enabled(c: &Comp, n: usize) -> bool {
    c.settings >> (n + 1) & 1 != 0
}

/// `ItemIndexLoader.method7171`: option n (0-based) is in the menu when the settings allow it or
/// the component has a `params` hook, and its text isn't blank.
fn in_menu(c: &Comp, n: usize) -> bool {
    (enabled(c, n) || c.hook("params").is_some()) && c.options.get(n).is_some_and(|t| !t.trim().is_empty())
}

/// The option (1..=10) this component would put on top of the menu, if any: the last one
/// `iComponentOnIComponent` adds going 10..6 then 5..1, which is the lowest one in the menu.
pub fn top_option(c: &Comp) -> Option<i32> {
    (0..10).find(|&n| in_menu(c, n)).map(|n| n as i32 + 1)
}

impl Ui {
    /// Click type 23 from the scene (`CutsceneArea.method12078` with 0): `WALK` to the tile,
    /// with the scene's base added. The run key byte (`VertexNormal.method1465`, a key the
    /// defaults name) is 0: that key isn't ported. Staff teleporting with it, the cross drawn
    /// where the click was and the client's own route for the flag (`MapRegion.routeTo`) aren't
    /// ported either.
    pub fn walk_here(&mut self, (x, y): (i32, i32)) {
        let Some(base) = self.players.as_ref().map(|p| p.base()) else { return };
        let (Ok(x), Ok(y)) = (u16::try_from(base.0 + x), u16::try_from(base.1 + y)) else { return };
        self.outgoing.push(client_packets::walk(false, x, y));
    }

    /// `method14603` for option `op` (1..=10) of component `r`.
    pub fn click_option(&mut self, r: CompRef, op: i32) {
        let Some(c) = self.comp(r) else { return };
        let params = c.hook("params").cloned();
        let allowed = usize::try_from(op - 1).is_ok_and(|n| enabled(c, n));
        if let Some(args) = params {
            run_hook(self, &Hook { source: Some(r), args, op, ..Default::default() }, 200_000);
        }
        if allowed {
            self.outgoing.push(client_packets::if_op(op as usize, r.id as i32, r.slot, -1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::interface::Kind;

    #[test]
    fn the_first_option_ends_on_top() {
        let mut c = Comp::created(1, Kind::Container, -1);
        c.options = vec!["World Select".into(), "".into(), "Two".into()];
        assert_eq!(top_option(&c), None); // no settings, no params hook
        c.settings = 0b1010; // options 1 and 3
        assert_eq!(top_option(&c), Some(1));
    }
}
