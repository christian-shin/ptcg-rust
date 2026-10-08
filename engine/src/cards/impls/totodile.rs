//! Totodile (TEF): Big Bite — 10; during your opponent's next turn, the
//! Defending Pokémon can't retreat (BLOCK_RETREAT: a PreventRetreatEffect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Totodile",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
