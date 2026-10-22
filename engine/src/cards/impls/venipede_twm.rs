//! Venipede (TWM 115): Spit Poison - your opponent's Active Pokémon is now Poisoned. Spinning Attack - 40.
//!
//! A GainCondition(Poisoned) with the attack's cause (Foongus BLK 10).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VenipedeTWM",
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
