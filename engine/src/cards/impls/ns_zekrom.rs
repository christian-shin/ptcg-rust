//! N's Zekrom (M2a / ASC): Shred - 70, not affected by effects on the
//! Defending Pokémon; Rampaging Thunder - 250, can't attack next turn.
//!
//! Twinleaf (phase 4b R7B): Shred sets `ignoreDefenderEffects`; the damage goes
//! through the normal path (Weakness, Resistance and the effects on the
//! attacker apply, the effects on the Defending Pokémon don't). It used to ignore
//! Resistance on the AttackEffect and add the damage straight to the Active.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsZekrom",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
