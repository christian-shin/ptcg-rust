//! Lampent (TWM 37): Live Coal — 20. Burn It All Up — 60; discard all Energy
//! from this Pokémon (DISCARD_ALL_ENERGY_FROM_POKEMON).
use super::mega_manectricex::discard_all_energy_from_pokemon;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LampentTWMPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        discard_all_energy_from_pokemon(g, e, me)?;
    }
    Ok(())
}
