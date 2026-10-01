//! Mega Charizard Y ex (ASC / MC): Explosion Y — discard 3 Energy from this
//! Pokémon, and 280 damage to 1 of your opponent's Pokémon (the Bench takes no
//! Weakness/Resistance).
//!
//! Twinleaf: DISCARD_X_ENERGY_FROM_THIS_POKEMON(3) queues its prompt, then
//! THIS_ATTACK_DOES_X_DAMAGE_TO_1_OF_YOUR_OPPONENTS_POKEMON queues the target
//! prompt without waiting.
use crate::cards::prelude::*;
use super::slither_wing::{discard_energy_chosen, discard_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl { class: "MegaCharizardYex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        discard_x_energy_from_this_pokemon(g, me, e, 3, 1)?;
        damage_1_opponent_pokemon(g, e, 280, false);
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_energy_chosen(g, f, results);
    }
    Ok(())
}
