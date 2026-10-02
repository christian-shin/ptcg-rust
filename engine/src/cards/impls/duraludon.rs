//! Duraludon (SCR): Hammer In — 30. Raging Hammer — 80+; 10 more damage for
//! each damage counter on this Pokémon.
//!
//! Twinleaf adds `player.active.damage` (the Active slot, not this card's own
//! slot) to the damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Duraludon@SCR|PRE", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let d = g.st.slot(p, a).damage;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += d;
        }
    }
    Ok(())
}
