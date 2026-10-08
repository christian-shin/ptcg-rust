//! Koraidon (TEF): Primordial Beatdown — 30× for each of your Ancient Pokémon
//! in play. Shred — 130; this attack's damage isn't affected by any effects
//! on your opponent's Active Pokémon.
//!
//! Twinleaf (temporal-forces file): Primordial Beatdown sets
//! `damage = 30 × Ancient Pokémon` (Active + Bench). Shred sets
//! `ignoreDefenderEffects` (phase 4b R7B: it used to add the damage straight to
//! the Active, skipping the attacker's effects).
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
