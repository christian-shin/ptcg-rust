//! Cubchoo (BLK 25): Snotted Up — 10; during your opponent's next turn, the Defending Pokémon can't use attacks.
//!
//! A lasting lock on the Defending Pokémon over its UseAttack (APR C-15), ending with the opponent's next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cubchoo",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_ATTACK)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
