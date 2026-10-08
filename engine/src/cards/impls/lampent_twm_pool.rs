//! Lampent (TWM 37): Live Coal — 20. Burn It All Up — 60; discard all Energy
//! from this Pokémon (DISCARD_ALL_ENERGY_FROM_POKEMON).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LampentTWMPool",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::This, selection: EnergySelection::AllProvided })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
