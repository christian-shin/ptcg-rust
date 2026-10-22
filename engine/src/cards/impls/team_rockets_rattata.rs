//! Team Rocket's Rattata (DRI 147): Dangerous Incisors - 10; your opponent's Active Pokémon is now Poisoned.
//!
//! A GainCondition(Poisoned) with the attack's cause (Gligar PFL 49).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsRattata",
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
