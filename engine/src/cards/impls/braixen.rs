//! Braixen (CRI 12): Flamethrower - 80; discard an Energy from this Pokémon.
//!
//! One Energy of any type chosen from this Pokémon after the damage (Charcadet SSP's Flamethrower).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Braixen",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
