//! Steven's Metang (DRI): Metal Slash — 70; during your next turn this
//! Pokémon can't attack (`cannotAttackNextTurnPending` on the Active).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "StevensMetang", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
