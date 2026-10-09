//! Shroodle (MEG 91): Poison Jab — 20; your opponent's Active Pokémon is now
//! Poisoned (an AddSpecialConditionsEffect [POISONED]).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ShroodleMEGPool",
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
