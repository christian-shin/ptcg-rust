//! Erika's Gloom (ASC 2): Poison Spray - 50; your opponent's Active Pokémon is now Poisoned.
//!
//! `inflict` Poisoned after the damage (Gligar PFL 49).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ErikasGloom",
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
