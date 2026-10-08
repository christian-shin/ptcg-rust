//! Shuppet (PBL / M5): Hide 'n' Sneak — prevent all effects of your
//! opponent's Pokémon's attacks and Abilities done to this Pokémon (damage is
//! not an effect). Hang Down — 10.
//!
//! Also hosts the shared `hide-n-sneak.ts` helpers (Banette, Poltchageist,
//! Sinistcha, Dhelmise).
//!
//! Twinleaf (`reduceHideNSneak`): every AbstractAttackEffect whose target
//! slot has this card on top (in play) is prevented unless it is
//! ApplyWeakness / PutDamage / DealDamage, when its `player` is not the
//! owner; the ability-lock probe (for the owner) runs before the owner check.
//! PlaceDamageCountersEffect is prevented when its source card sits in a slot
//! owned by the effect's player. (AddSpecialConditionsPowerEffect and
//! PutDamageCountersEffect branches: no ported card emits them yet.)
use crate::spec::prelude::*;

// The other Hide 'n' Sneak Pokémon share these.
pub use crate::spec::passive::{count_hide_n_sneak_in_discard, reduce_hide_n_sneak, HIDE_N_SNEAK_KINDS};

pub static SPEC: CardSpec = CardSpec {
    class: "Shuppet",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::PreventAttackEffects(HIDE_N_SNEAK) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
