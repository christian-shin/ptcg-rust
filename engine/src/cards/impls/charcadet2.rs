//! Charcadet (SSP 33, "Charcadet 2"): Light Punch — 10. Flamethrower — 70;
//! discard an Energy from this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Charcadet2",
    // Flamethrower: discard an Energy from this Pokémon.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::Choose { count: 1, ty: ct::COLORLESS } }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
