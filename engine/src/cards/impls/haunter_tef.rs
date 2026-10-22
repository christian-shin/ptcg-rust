//! Haunter (TEF 103): Super Poison Breath — 30; your opponent's Active Pokémon is now Poisoned.
//!
//! A Special Condition event on the Defending Pokémon after the damage (an effect of the attack: Mist Energy prevents it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HaunterTEF",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Poisoned]))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
