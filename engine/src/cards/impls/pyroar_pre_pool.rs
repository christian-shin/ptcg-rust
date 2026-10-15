//! Pyroar (PRE 16): Fire Mane - 50. Flame Tackle - 160; during your next turn this Pokémon can't attack.
//!
//! Flame Tackle arms a `CANT_ATTACK` lasting lock on this Pokémon until your next turn (any attack).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PyroarPRE",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
