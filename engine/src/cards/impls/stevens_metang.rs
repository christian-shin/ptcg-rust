//! Steven's Metang (DRI 144): Metal Slash — 70; during your next turn this Pokémon can't attack.
//!
//! A lasting lock on this Pokémon over its UseAttack until the end of your next turn (`Lasting::Lock`, bound to the attacker, not to
//! the Defending Pokémon); it goes when the Pokémon moves to the Bench or evolves (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensMetang",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
