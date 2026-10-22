//! Garbodor (WHT 60): Suffocating Gas - 40. Gunk Shot - 120; your opponent's Active Pokémon is now Poisoned.
//!
//! `inflict` Poisoned after the damage (Gligar PFL 49).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Garbodor",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Poisoned])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
