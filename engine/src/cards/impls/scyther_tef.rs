//! Scyther (TEF): Cut Up — 10. Slashing Strike — 60; during your next turn
//! this Pokémon can't use Slashing Strike.
//!
//! THIS_POKEMON_CANNOT_USE_THIS_ATTACK_NEXT_TURN: push the name onto the
//! player's Active `cannotUseAttacksNextTurnPending` if missing.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Scyther@TEF", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 1, me) {
        return Ok(());
    }
    if let Effect::Attack { p, .. } = *g.e(e) {
        let p = p as usize;
        let a = g.st.players[p].active;
        let pending = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
        if !pending.iter().any(|n| *n == "Slashing Strike") {
            pending.push("Slashing Strike");
        }
    }
    Ok(())
}
