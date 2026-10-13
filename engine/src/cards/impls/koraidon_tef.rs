//! Koraidon (TEF): Primordial Beatdown - 30x for each of your Ancient Pokémon in play. Shred - 130; this attack's damage isn't
//! affected by any effects on your opponent's Active Pokémon.
//!
//! Primordial Beatdown sets the damage to 30 times the Ancient Pokémon you have in play (Active and Bench). Shred sets the attack
//! flag `IgnoreDefenderEffects` before the damage: the damage skips the defending side's modifiers and preventions and keeps the
//! attacker's own (APR C-16, RULES.md "Shred").
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Koraidon@TEF",
    attacks: &[
        // Primordial Beatdown: 30 for each of your Ancient Pokémon in play.
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(damage_is(Num::Mul(&Num::InPlayCount(Who::Me, PlayScope::All, Pred::Tag(tag::ANCIENT)), &Num::Lit(30))))],
        },
        // Shred: effects on the Defending Pokémon don't change the damage.
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
