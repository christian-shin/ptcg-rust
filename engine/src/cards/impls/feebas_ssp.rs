//! Feebas (SSP): Leap Out — switch this Pokémon with 1 of your Benched Pokémon.
//!
//! An attack effect after the damage: `switch_self`. The class is pinned to SSP: Feebas TWM is another card with the same
//! name.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FeebasSSP",
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
