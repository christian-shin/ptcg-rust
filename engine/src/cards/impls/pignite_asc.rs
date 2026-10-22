//! Pignite (ASC 30): Super Singe - 70; your opponent's Active Pokémon is now Burned.
//!
//! Burn by `inflict` after the damage (Darmanitan BLK 14).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PigniteASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Burned])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
