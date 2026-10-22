//! Groudon (ASC 108): Hammer In - 80. Megaton Fall - 150; this Pokémon also does 30 damage to itself.
//!
//! One Damage event on the attacker with the attack as its cause, after the main damage (Arrokuda DRI 62).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Groudon",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
