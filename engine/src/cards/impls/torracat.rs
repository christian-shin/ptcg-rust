//! Torracat (TEF, support card): Bite — 30. Flare Strike — 80; during your
//! next turn, this Pokémon can't use Flare Strike.
//!
//! THIS_POKEMON_CANNOT_USE_THIS_ATTACK_NEXT_TURN: push the name onto the
//! player's Active `cannotUseAttacksNextTurnPending` if missing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Torracat@TEF",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
