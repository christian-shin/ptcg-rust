//! Light Ball (ASC, tool): attacks used by the Pikachu ex this card is
//! attached to do 50 more damage to your opponent's Active Pokémon ex
//! (before applying Weakness and Resistance).
//!
//! Twinleaf: on DealDamageEffect from the holder's slot: the holder must be
//! named 'Pikachu ex', then the tool probe, then the target must be the
//! opponent's Active; +50 when that is an ex and the current damage is
//! above 0.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "LightBall",
    passives: &[
        Passive { origin: RuleSource::Tool, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 50, attacker: SlotPred::All(&[SlotPred::Holder, SlotPred::Named("Pikachu ex")]), target: SlotPred::Tag(tag::POKEMON_EX_LOWER), needs_damage: true, ..DamageDealtSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
