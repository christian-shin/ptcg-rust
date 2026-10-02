//! Pyroar (PRE): Fire Mane - 50. Flame Tackle - 160; during your next turn
//! this Pokémon can't attack.
//!
//! Twinleaf: THIS_POKEMON_CANNOT_ATTACK_NEXT_TURN sets
//! `cannotAttackNextTurnPending` on the player's Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PyroarPREPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let pl = &mut g.st.players[p as usize];
            let a = pl.active;
            pl.slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
