//! Girafarig (TEF): Psychic Assault — 20+, 10 more for each damage counter
//! on your opponent's Active Pokémon (Twinleaf adds `opponent.active.damage`).
//! Ported so Farigiraf ex (TEF) can evolve in check decks.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Girafarig@TEF", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let o = match *g.e(e) {
        Effect::Attack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    let d = g.st.slot(o, g.st.players[o].active).damage;
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage += d;
    }
    Ok(())
}
