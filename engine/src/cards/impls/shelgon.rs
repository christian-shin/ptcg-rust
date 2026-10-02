//! Shelgon (JTG, support card): Guard Press — 30; during your opponent's next
//! turn, this Pokémon takes 30 less damage from attacks (after applying
//! Weakness and Resistance). Heavy Impact — 80.
//!
//! Twinleaf sets `player.active.damageReductionNextTurn = 30` directly.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Shelgon@JTG", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

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
