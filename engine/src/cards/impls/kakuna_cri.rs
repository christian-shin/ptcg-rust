//! Kakuna (CRI / M4): Exoskeleton — this Pokémon takes 20 less damage from
//! attacks. Hang Down — 20.
//!
//! Twinleaf: applied on DealDamageEffect (before Weakness/Resistance,
//! despite the text) whenever this card is in the target's list; the lock
//! probe runs before the top-Pokémon check.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Kakuna@CRI", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::DealDamage { b, .. } => b,
        _ => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).cards.contains(me) {
        return Ok(());
    }
    if is_ability_blocked(g, t.p as usize, me, None) {
        return Ok(());
    }
    if g.st.slot_pokemon(t.p as usize, t.s) == Some(me) {
        if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
            *damage = (*damage - 20).max(0);
        }
    }
    Ok(())
}
