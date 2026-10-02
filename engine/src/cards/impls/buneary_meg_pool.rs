//! Buneary (MEG 107): Charm — during your opponent's next turn, attacks used
//! by the Defending Pokémon do 20 less damage (before W/R). Skip — 10.
//!
//! DEFENDING_POKEMON_DOES_LESS_DAMAGE (shared with Chikorita ASC).
use super::chikorita_asc::defending_pokemon_does_less_damage;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BunearyMEGPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return defending_pokemon_does_less_damage(g, e, 20);
    }
    Ok(())
}
