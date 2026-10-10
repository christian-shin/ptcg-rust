//! Electrike (MEG 49): Thunder Jolt — 30; this Pokémon also does 10 damage to
//! itself.
//!
//! Rule: after the damage this Pokémon takes a Damage event of 10 (cause: this attack):
//! the attacker's DealDamage modifiers apply, Weakness and Resistance don't (it isn't the
//! opponent's Active Pokémon, APR B-08).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Electrike@MEG",
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
