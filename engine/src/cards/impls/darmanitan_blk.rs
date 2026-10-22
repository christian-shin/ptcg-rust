//! Darmanitan (BLK 14): Searing Flame - 50; your opponent's Active Pokémon is now Burned. Smashing Headbutt - 180; discard 2 Energy from this Pokémon.
//!
//! A Burned condition after the damage (`inflict`); two Energy of any type chosen from this Pokémon after the damage (Bouffalant SSP).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DarmanitanBLK",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Burned])),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
