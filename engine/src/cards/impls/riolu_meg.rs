//! Riolu (ASC 112, M1L / MEG 76): Accelerating Stab - 30. During your next turn, this Pokémon can't use Accelerating Stab.
//!
//! The attack arms a lasting lock on this Pokémon that names the attack (`CANT_ATTACK` naming `NamedAttack::This`) until your
//! next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RioluMEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
