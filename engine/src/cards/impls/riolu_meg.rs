//! Riolu (M1L / MEG 76): Accelerating Stab — 30. During your next turn, this
//! Pokémon can't use Accelerating Stab.
//!
//! Twinleaf has several `Riolu` classes; this port is bound to M1L.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Riolu@Riolu M1L|Riolu ASC",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotUseThisAttackNextTurn }))] }],
    ..CardSpec::NONE
};

// Mega Lucario ex uses the same lock.
pub use crate::spec::ops::state::push_cannot_use_attack as push_pending;

pub static IMPL: CardImpl = SPEC.card_impl();
