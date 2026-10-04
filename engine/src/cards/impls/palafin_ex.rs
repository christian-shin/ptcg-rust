//! Palafin ex (TWM): Hero's Spirit — can only be put into play with Palafin's
//! Zero to Hero. Giga Impact — 250; during your next turn, this Pokémon
//! can't attack.
//!
//! Twinleaf: any EvolveEffect for this card throws CANNOT_EVOLVE (Zero to Hero
//! moves it into play without one). It used to be lifted when the generic
//! ability-lock probe said blocked, which the Iron Thorns ex hand lock (phase
//! 4b) would have turned into a way to evolve it from the hand.
//! The TWM support print and Palafin exSAR PRE share the same Palafinex logic.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Palafinex", mask: mask(&[k::EVOLVE, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::Evolve { card, .. } = *g.e(e) {
        if card == me {
            bail!("CANNOT_EVOLVE");
        }
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
