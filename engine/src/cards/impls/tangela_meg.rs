//! Tangela (MEG 6): Poison Powder - your opponent's Active Pokémon is now Poisoned. Hook - 30.
//!
//! A GainCondition(Poisoned) with the attack's cause (Foongus BLK 10's Toxic Spore).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TangelaMEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Poisoned])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
