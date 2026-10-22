//! Skuntank (CRI 54): Rear Kick - 40. Smash Turn - 100; switch this Pokémon with 1 of your Benched Pokémon.
//!
//! `switch_self` after the damage (Cascoon ASC 14).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Skuntank",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(switch_self()),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
