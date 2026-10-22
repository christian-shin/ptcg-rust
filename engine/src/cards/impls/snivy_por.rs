//! Snivy (POR 4): Reckless Charge - 30; this Pokémon also does 10 damage to itself.
//!
//! One Damage event on the attacker after the main damage (Arrokuda DRI 62).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SnivyPOR",
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
