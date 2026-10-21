//! Arven's Toedscool (DRI 109): Slight Intrusion - 30; this Pokémon also does 10 damage to itself.
//!
//! The self-damage is one Damage event on the attacker with the attack as its cause, after the main damage (the same step as
//! Beldum TEF 113's Iron Tackle).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ArvensToedscool",
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
