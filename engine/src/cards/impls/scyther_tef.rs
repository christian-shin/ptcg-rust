//! Scyther (TEF): Cut Up — 10. Slashing Strike — 60; during your next turn
//! this Pokémon can't use Slashing Strike.
//!
//! THIS_POKEMON_CANNOT_USE_THIS_ATTACK_NEXT_TURN: push the name onto the
//! player's Active `cannotUseAttacksNextTurnPending` if missing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Scyther@TEF",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotUseThisAttackNextTurn }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
