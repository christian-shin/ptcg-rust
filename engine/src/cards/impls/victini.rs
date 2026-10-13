//! Victini (SSP 21): Victory Cheer — attacks used by your Evolution [R] Pokémon do 10 more damage to your opponent's Active Pokémon
//! (before applying Weakness and Resistance). Flare — 30.
//!
//! A `DamageDealt` modifier of 10 over the attacks of your Evolution Pokémon that are [R] (read after effects), added before
//! Weakness and Resistance; one per Victini (two stack, +20), and only while its Ability works.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Victini",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 10,
            attacker: SlotPred::All(&[SlotPred::TypeIs(ct::FIRE), SlotPred::Evolution]),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
