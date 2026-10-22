//! Pupitar (JTG 81): Take Down - 60; this Pokémon also does 20 damage to itself.
//!
//! Self damage after the damage (Klang SCR 100).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PupitarJTG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(self_damage(20)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
