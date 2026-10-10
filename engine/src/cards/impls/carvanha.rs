//! Carvanha (M2 / PFL 60): Reckless Charge — 30, this Pokémon also does 10 damage to
//! itself.
//!
//! The self-damage is one Damage event on the attacker with the attack as its cause, after the main damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Carvanha",
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
