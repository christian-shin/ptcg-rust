//! Finizen (TWM): Aqua Slash — 30; during your next turn, this Pokémon can't
//! attack (`player.active.cannotAttackNextTurnPending = true`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Finizen",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
