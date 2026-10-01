//! Zeraora (SCR): Combat Thunder — 20+, 20 more for each of your opponent's
//! Benched Pokémon.
//!
//! Twinleaf (stellar-crown file): sets `effect.damage = 20 + 20 × occupied
//! opponent bench slots` (overwrites earlier modifications).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Zeraora@SCR", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let o = match *g.e(e) {
        Effect::Attack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    let pl = &g.st.players[o];
    let n = pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count() as i32;
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage = 20 + n * 20;
    }
    Ok(())
}
