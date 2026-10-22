//! Excadrill (TEF 86): Mud Shot - 20. Wild Tackle - 180; this Pokémon also does 50 damage to itself.
//!
//! Self damage after the damage (Arrokuda DRI 62).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Excadrill",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(50)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
