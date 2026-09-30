//! Chi-Yu (M5): Whirling Envy — 20+, 90 more if your Active Pokémon has 2 or
//! more damage counters; not affected by Weakness.
//!
//! Twinleaf checks `player.active.damage` (not necessarily this Pokémon).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ChiYu@Chi-Yu M5", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let dmg = g.st.slot(p, g.st.players[p].active).damage;
    if let Effect::Attack { damage, ignore_weakness, .. } = g.e_mut(e) {
        *ignore_weakness = true;
        if dmg >= 20 {
            *damage += 90;
        }
    }
    Ok(())
}
