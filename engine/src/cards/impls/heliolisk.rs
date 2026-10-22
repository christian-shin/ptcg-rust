//! Heliolisk (MEG 53): Dazzle Blast - 20; your opponent's Active Pokémon is now Confused. Head Bolt - 70.
//!
//! `inflict` Confused after the damage (Alakazam TWM 82).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Heliolisk",
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
