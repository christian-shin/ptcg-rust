//! Varoom (SFA): Rigidify — during your opponent's next turn this Pokémon
//! takes 30 less damage (`damageReductionNextTurn = 30` on the Active).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Varoom", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].damage_reduction_next_turn = 30;
        }
    }
    Ok(())
}
