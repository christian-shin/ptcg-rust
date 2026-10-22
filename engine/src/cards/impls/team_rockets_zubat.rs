//! Team Rocket's Zubat (DRI 120): Poison Spray - your opponent's Active Pokémon is now Poisoned.
//!
//! A GainCondition(Poisoned) with the attack's cause (Foongus BLK 10).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsZubat",
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
