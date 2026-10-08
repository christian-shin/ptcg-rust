//! Pyroar (PRE): Fire Mane - 50. Flame Tackle - 160; during your next turn
//! this Pokémon can't attack.
//!
//! Twinleaf: THIS_POKEMON_CANNOT_ATTACK_NEXT_TURN sets
//! `cannotAttackNextTurnPending` on the player's Active.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PyroarPREPool",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
