//! Component options and the left click. The reference builds a right-click menu each tick from
//! the components under the mouse (`Class455.iComponentOnIComponent`: options 10..6, then 5..1,
//! each added on top) and a left click runs the top entry. Only that top entry is kept here:
//! the menu itself (drawing it, right-click, cancel, the other entry kinds) isn't ported yet.
//!
//! Running an option (`CutsceneAction_Sub10.method14603`): the component's `params` hook runs at
//! once with the option number, then, if its settings allow that option, `IF_OPn` goes to the
//! server.

use super::component::Comp;
use super::{CompRef, Ui};
use crate::client::cs2::{run_hook, Hook};
use crate::login::client_prot;

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
    /// `method14603` for option `op` (1..=10) of component `r`.
    pub fn click_option(&mut self, r: CompRef, op: i32) {
        let Some(c) = self.comp(r) else { return };
        let params = c.hook("params").cloned();
        let allowed = usize::try_from(op - 1).is_ok_and(|n| enabled(c, n));
        if let Some(args) = params {
            run_hook(self, &Hook { source: Some(r), args, op, ..Default::default() }, 200_000);
        }
        if allowed {
            let id = client_prot::IF_OPS[(op - 1) as usize];
            self.outgoing.push((id, client_prot::if_op_payload(r.id as i32, r.slot, -1)));
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
