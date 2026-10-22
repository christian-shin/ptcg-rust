//! Aegislash (POR 58): Slash — 80. Metal Slash — 230; during your next turn, this Pokémon can't use attacks.
//!
//! Metal Slash arms a lasting lock on every attack of this Pokémon through your next turn (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AegislashPOR",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
