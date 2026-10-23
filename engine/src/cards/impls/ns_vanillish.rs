//! N's Vanillish (ASC 50): Flop - 20. Sheer Cold - 60; during your opponent's next turn, the Defending Pokémon can't use
//! attacks.
//!
//! The lasting `CANT_ATTACK` lock on the Defending Pokémon until the end of the opponent's next turn (Cubchoo BLK 25).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsVanillish",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_ATTACK)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
