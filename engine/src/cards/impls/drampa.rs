//! Drampa (ASC 176): Gentle Slap - 50. Dragon Strike - 120; during your next turn, this Pokémon can't use Dragon Strike.
//!
//! A lasting lock on this Pokémon over the named attack (APR C-15), through the end of your next turn (Scyther TEF).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Drampa",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
