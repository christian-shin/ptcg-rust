//! Torracat (TEF): Bite — 30. Flare Strike — 80; during your next turn, this Pokémon can't use Flare Strike.
//!
//! A lasting lock on this Pokémon naming this attack (`NamedAttack::This`) until the end of your next turn; its other attack stays
//! usable, and the lock goes when the Pokémon moves to the Bench or evolves (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Torracat@TEF",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
