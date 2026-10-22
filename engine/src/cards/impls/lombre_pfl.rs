//! Lombre (PFL 6): Mega Drain - 30; heal 30 damage from this Pokémon.
//!
//! `heal_active` after the damage (Beheeyem BLK 41).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LombrePFL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
