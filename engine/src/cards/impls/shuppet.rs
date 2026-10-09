//! Shuppet (PBL / M5): Hide 'n' Sneak — prevent all effects of your
//! opponent's Pokémon's attacks and Abilities done to this Pokémon (damage is
//! not an effect). Hang Down — 10.
//!
//! Also hosts the shared `hide-n-sneak.ts` helpers (Banette, Poltchageist,
//! Sinistcha, Dhelmise).
//!
//! Rule: every effect of an opponent's attack or Ability done to this Pokémon
//! is prevented, damage excepted. This covers the Special Condition events an
//! attack or Ability causes (GainCondition with that cause) and the placing of
//! damage counters, when the effect's player is not the owner; the ability-lock
//! probe (for the owner) runs before the owner check.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Shuppet",
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::PreventAttackEffects(HIDE_N_SNEAK) },
        // The opponent's attacks and Abilities switching this Pokémon in or out (ChangeActive; JP Q&A, Hariyama MEG 73).
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(HIDE_N_SNEAK_SWITCH) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
