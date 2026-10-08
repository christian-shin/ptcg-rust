//! Carvanha (M2 / PFL 60): Assault — 30, this Pokémon also does 10 damage to
//! itself.
//!
//! Twinleaf: a DealDamageEffect(effect, 10) targeting the attacker's current
//! Active, reduced during the AttackEffect (before the main damage).
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
