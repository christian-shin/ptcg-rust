//! Pikachu ex (PRE 28): Tail Whap — 30. Thunder — 220; this Pokémon also does 30 damage to itself.
//!
//! The self-damage is a Damage event on this Pokémon caused by the attack, with Weakness, Resistance and the effects on it
//! (APR B-09).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PikachuExPRE",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(self_damage(30))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
