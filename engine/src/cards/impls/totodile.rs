//! Totodile (TEF 39): Big Bite — 10; during your opponent's next turn, the Defending Pokémon can't retreat.
//!
//! A retreat lock on the Defending Pokémon until the end of the opponent's next turn (`Lasting::Lock`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Totodile",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
