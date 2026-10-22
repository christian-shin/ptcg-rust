//! Spewpa (SSP 6): Wander About - switch this Pokémon with 1 of your Benched Pokémon. Tackle - 30.
//!
//! `switch_self` (Cascoon ASC 14).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SpewpaSSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(switch_self()),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
