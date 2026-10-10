//! Riolu (M1L / MEG 76): Accelerating Stab — 30. During your next turn, this
//! Pokémon can't use Accelerating Stab.
//!
//! Twinleaf has several `Riolu` classes; this port is bound to M1L.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Riolu@MEG|ASC",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
