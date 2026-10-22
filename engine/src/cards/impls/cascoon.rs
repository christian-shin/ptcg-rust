//! Cascoon (ASC 14): Trading Places - switch this Pokémon with 1 of your Benched Pokémon.
//!
//! The attack's one effect is `switch_self` (Feebas SSP's Leap Out); it has no damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cascoon",
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
