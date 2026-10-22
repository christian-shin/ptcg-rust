//! Team Rocket's Drowzee (DRI 79): Hypnotic Ray - 10; your opponent's Active Pokémon is now Asleep.
//!
//! A GainCondition(Asleep) on the Defending Pokémon with the attack's cause, after the damage (Shroodle MEG 91's Poison).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsDrowzee",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Asleep])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
