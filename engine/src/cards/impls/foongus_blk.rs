//! Foongus (BLK 10): Toxic Spore - your opponent's Active Pokémon is now Poisoned.
//!
//! A GainCondition(Poisoned) with the attack's cause (Shroodle MEG 91).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Foongus",
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
