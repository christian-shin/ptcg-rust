//! Duraludon (SSP 129): Confront — 50. Duralubeam — 130; discard 2 Energy
//! from this Pokémon.
//!
//! Twinleaf: ChooseEnergyPrompt over the Active's CheckProvidedEnergy map for
//! [C][C] (no cancel), then a DiscardCardsEffect aimed at `player.active`
//! (the same shape as DISCARD_X_ENERGY_FROM_THIS_POKEMON).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Duraludon@SSP",
    // Duralubeam: discard 2 Energy from this Pokémon.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS } }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
