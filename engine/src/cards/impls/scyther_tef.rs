//! Scyther (TEF 1): Cut Up - 10. Slashing Strike - 60; during your next turn this Pokémon can't use Slashing Strike.
//!
//! Slashing Strike arms a lasting lock on this Pokémon that names the attack (`CANT_ATTACK` naming `NamedAttack::This`) until
//! your next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Scyther",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
