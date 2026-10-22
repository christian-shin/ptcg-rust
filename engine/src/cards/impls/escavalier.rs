//! Escavalier (BLK 60): Wild Lances - 90; this Pokémon also does 30 damage to itself.
//!
//! Self damage after the damage (Arrokuda DRI 62).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Escavalier",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
