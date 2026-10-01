//! Blaziken (DRI): Heat Blast — 70. Inferno Legs — 120; discard 2 Energy from
//! this Pokémon, and 120 damage to 1 of your opponent's Benched Pokémon.
use crate::cards::prelude::*;
use super::slither_wing::{discard_energy_chosen, discard_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl { class: "Blaziken", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        discard_x_energy_from_this_pokemon(g, me, e, 2, 1)?;
        damage_1_opponent_pokemon(g, e, 120, true);
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_energy_chosen(g, f, results);
    }
    Ok(())
}
