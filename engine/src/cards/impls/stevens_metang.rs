//! Steven's Metang (DRI): Metal Slash — 70; during your next turn this
//! Pokémon can't attack (`cannotAttackNextTurnPending` on the Active).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensMetang",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
