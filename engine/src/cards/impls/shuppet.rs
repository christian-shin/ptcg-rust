//! Shuppet (PBL / M5): Hide 'n' Sneak — prevent all effects of your
//! opponent's Pokémon's attacks and Abilities done to this Pokémon (damage is
//! not an effect). Hang Down — 10.
//!
//! `Modifier::Prevent(HIDE_N_SNEAK)` (also Banette, Poltchageist, Sinistcha, Dhelmise): one declaration over every event
//! with an effect (`EFFECT_EVENT_KINDS`) whose cause is an attack or an Ability of the opponent's Pokémon: Special
//! Conditions, damage counters placed or moved onto it, discards, switches, devolving, lasting effects (APR C-17), a
//! Knock Out by an effect; never an attack's damage (APR C-17) nor what the damage records. It is an Ability: when it is
//! off nothing is prevented.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Shuppet",
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(HIDE_N_SNEAK) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
