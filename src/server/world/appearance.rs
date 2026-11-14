//! How a player looks to the others: what the appearance mask of `PLAYER_UPDATE` carries. A new
//! player's look is this server's choice, taken from the reference server (`Appearance.male`,
//! `buildSlots`, `Equipment.weaponBAS`), and so is the combat level formula
//! (`Skills.getCombatLevel`).

use crate::protocol::player_update::Appearance;

/// The reference server's default male look (`Appearance.male`): `lookI` hair 310, beard 16,
/// chest 452, arms −1, hands 371, legs 627, feet 433.
const HAIR: u16 = 310;
const BEARD: u16 = 16;
const CHEST: u16 = 452;
const HANDS: u16 = 371;
const LEGS: u16 = 627;
const FEET: u16 = 433;

/// Its colours (`colour[0..5]` = 12, 218, 218, 180, 110, the rest 0).
const COLOURS: [u8; 10] = [12, 218, 218, 180, 110, 0, 0, 0, 0, 0];

/// The base animations with no weapon: the reference server falls back to 1426
/// (`PKBotNPC`: `getRenderAnimId() ?: 1426`, the same rule as `Equipment.weaponBAS`).
const UNARMED_RENDER: u16 = 1426;

/// A body style in a slot: 0x100 + the style (`Rev727AppearanceEncoder.BODY_STYLE_OFFSET`).
const fn body(style: u16) -> u16 {
    0x100 + style
}

/// The slots in the order the reference server sends them for a player wearing nothing
/// (`buildSlots`): head, cape, neck, weapon (empty), chest, shield (empty), arms (−1: empty),
/// legs, hair, hands, feet, beard, aura (empty). These are the 13 the cache's equipment table
/// doesn't hide.
const SLOTS: [u16; 13] = [0, 0, 0, 0, body(CHEST), 0, 0, body(LEGS), body(HAIR), body(HANDS), body(FEET), body(BEARD), 0];

/// The reference server's combat level (`Skills.getCombatLevel(true)`, outside PvP): the best of
/// attack + strength, 1.5 × ranged and 1.5 × magic (each truncated), × 1.3 truncated, then
/// (that + defence + constitution + prayer / 2 + summoning / 2) / 4. Levels by skill id.
pub fn combat_level(level: impl Fn(u8) -> u8) -> u8 {
    let l = |skill| i32::from(level(skill));
    let (attack, defence, strength, constitution, ranged, prayer, magic, summoning) = (l(0), l(1), l(2), l(3), l(4), l(5), l(6), l(23));
    let base = (attack + strength).max(ranged * 3 / 2).max(magic * 3 / 2);
    let base = (f64::from(base) * 1.3) as i32;
    ((base + defence + constitution + prayer / 2 + summoning / 2) / 4) as u8
}

/// A new player's appearance: the default male look, their name, their combat level; outside
/// PvP the summoning combat level is 0 and the PvP threshold −1 (255), as the reference sends.
pub fn new_player(name: &str, combat_level: u8) -> Appearance {
    Appearance {
        female: false,
        prefix_title: None,
        postfix_title: None,
        slots: SLOTS.to_vec(),
        colours: COLOURS,
        render_emote: UNARMED_RENDER,
        name: name.to_string(),
        combat_level,
        summoning_combat_level: 0,
        pvp_threshold: 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every skill 1 but constitution 10: (2 × 1.3 → 2, + 1 + 10 + 0 + 0) / 4 = 3.
    #[test]
    fn a_new_player_is_combat_level_3() {
        assert_eq!(combat_level(|skill| if skill == 3 { 10 } else { 1 }), 3);
        assert_eq!(combat_level(|_| 99), 138);
    }
}
