//! Zubat (CRI 49): Supersonic - your opponent's Active Pokémon is now Confused.
//!
//! A GainCondition(Confused) with the attack's cause (Foongus BLK 10).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ZubatCRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Confused])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
