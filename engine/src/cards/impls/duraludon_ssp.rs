//! Duraludon (SSP 129): Confront — 50. Duralubeam — 130; discard 2 Energy
//! from this Pokémon.
//!
//! Twinleaf: ChooseEnergyPrompt over the Active's CheckProvidedEnergy map for
//! [C][C] (no cancel), then a DiscardCardsEffect aimed at `player.active`
//! (the same shape as DISCARD_X_ENERGY_FROM_THIS_POKEMON).
use crate::cards::prelude::*;
use super::slither_wing::{discard_energy_chosen, discard_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl { class: "Duraludon@SSP", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        discard_x_energy_from_this_pokemon(g, me, e, 2, 1)?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_energy_chosen(g, f, results);
    }
    Ok(())
}
