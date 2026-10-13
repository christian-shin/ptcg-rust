//! N's Zekrom (ASC 155): Shred - 70, this attack's damage isn't affected by any effects on your opponent's Active Pokémon;
//! Rampaging Thunder - 250, during your next turn this Pokémon can't use attacks.
//!
//! Shred sets the `IgnoreDefenderEffects` attack flag: the damage goes through the normal calculation (Weakness, Resistance and the
//! effects on the attacker apply; the effects on the Defending Pokémon don't: RULES.md "Shred", APR C-16). Rampaging Thunder
//! arms a `CANT_ATTACK` lasting lock on this Pokémon until your next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsZekrom",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
