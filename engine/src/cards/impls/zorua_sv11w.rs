//! Zorua (SV11W / WHT 61): Take Down — 30; this Pokémon also does 10 damage
//! to itself.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zorua@WHT",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(self_damage(10)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
