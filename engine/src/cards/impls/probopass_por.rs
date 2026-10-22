//! Probopass (POR 38): Rolling Rocks - 60. Obliterating Nose - 260; discard 3 Energy from this Pokémon.
//!
//! The discard of 3 chosen Energy after the damage (Darmanitan BLK 14 discards 2).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ProbopassPOR",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 3, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
