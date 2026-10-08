//! Dhelmise (JTG 70): Bind Down — 60; during your opponent's next turn the
//! Defending Pokémon can't retreat. Anchor Smash — 130.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DhelmiseJTGPool",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
