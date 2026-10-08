//! Rellor (TEF): Slight Intrusion — 30; this Pokémon also does 10 damage to
//! itself.
//!
//! Twinleaf: the self-damage is a DealDamageEffect on the player's Active,
//! reduced during the AttackEffect (before the attack's own damage).
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
