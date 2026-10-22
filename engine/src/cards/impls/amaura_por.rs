//! Amaura (POR 23): Icy Wind — 50; your opponent's Active Pokémon is now Asleep.
//!
//! A Special Condition event on the Defending Pokémon after the damage (an effect of the attack: Mist Energy prevents it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AmauraPOR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Asleep]))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
