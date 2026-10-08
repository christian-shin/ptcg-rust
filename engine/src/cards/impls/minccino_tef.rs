//! Minccino (TEF): Beat — 10. Cleaning Up — discard up to 2 Pokémon Tools
//! from your opponent's Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Minccino@TEF",
    attacks: &[
        AttackSpec { index: 0, steps: &[] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: OPP_ACTIVE, selection: EnergySelection::OppTools { max: 2 } }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
