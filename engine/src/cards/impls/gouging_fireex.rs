//! Gouging Fire ex (TEF 38): Heat Blast — 60. Blaze Blitz — 260; this Pokémon can't use Blaze Blitz again until it leaves the
//! Active Spot.
//!
//! Blaze Blitz arms a lasting lock on this Pokémon (`CANT_USE_ATTACK_AGAIN`, naming this attack, until it leaves the Active
//! Spot); it is armed on the attacker, so effects on the Defending Pokémon (Empoleon ex, Mist Energy) don't stop it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GougingFireEx",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_USE_ATTACK_AGAIN, LockUntil::LeavesActive).naming(NamedAttack::This)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
