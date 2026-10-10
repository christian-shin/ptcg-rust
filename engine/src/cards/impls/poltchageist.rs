//! Poltchageist (PBL 5): Hide 'n' Sneak — prevent all effects of your opponent's
//! Pokémon's attacks and Abilities done to this Pokémon. (Damage is not an effect.)
//! Furtive Drop — place 1 damage counter on your opponent's Active Pokémon.
//!
//! Rule: Hide 'n' Sneak is `Prevent(HIDE_N_SNEAK)`: one declaration over every event with an
//! effect (counters placed or moved onto it, Special Conditions, switches, discards, ...) whose
//! cause is an attack or an Ability of the opponent's Pokémon, never Damage (APR C-17). Furtive
//! Drop is a PlaceCounters event of 1 (cause: this attack; no Weakness or Resistance, APR C-07),
//! refused by Hide 'n' Sneak, Mist Energy, and nothing else on the Active Pokémon (Battle Cage
//! protects the Bench only).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Poltchageist",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(OPP_ACTIVE), counters: Num::Lit(1) }))],
    }],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(HIDE_N_SNEAK) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
