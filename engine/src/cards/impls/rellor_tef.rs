//! Rellor (TEF): Slight Intrusion — 30; this Pokémon also does 10 damage to itself.
//!
//! The self damage is a Damage event caused by the attack on this Pokémon (no Weakness or Resistance, APR B-08).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rellor@TEF",
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
