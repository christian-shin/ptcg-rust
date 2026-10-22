//! N's Klang (JTG 104): Spinning Gears - 20; your opponent's Active Pokémon is now Confused. Confront - 40.
//!
//! Confusion by `inflict` after the damage (Darmanitan BLK 14 does Burned).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsKlang",
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
