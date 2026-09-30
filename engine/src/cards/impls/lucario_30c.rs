//! Lucario (30C): Aura Sphere — 100, and 60 damage to 1 of your opponent's
//! Benched Pokémon.
//!
//! THIS_ATTACK_DOES_X_DAMAGE_TO_1_OF_YOUR_OPPONENTS_BENCHED_POKEMON, only
//! when the opponent has a Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Lucario@30C", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        damage_1_opponent_pokemon(g, e, 60, true);
    }
    Ok(())
}
