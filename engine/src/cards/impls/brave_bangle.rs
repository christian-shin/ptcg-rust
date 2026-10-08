//! Brave Bangle (SV11W, tool): if the holder doesn't have a Rule Box, its
//! attacks do 30 more damage to the opponent's Active Pokémon ex (before W/R).
//!
//! Twinleaf: the tool block probe is a bare ToolEffect; the Rule Box check
//! is on the attacking slot's cards; the attack's printed damage must be
//! above 0.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "BraveBangle",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 30,
            attacker: SlotPred::All(&[SlotPred::Holder, SlotPred::Not(&SlotPred::RuleBox)]),
            side: Side::Any,
            target: SlotPred::Tag(tag::POKEMON_EX_LOWER),
            needs_printed_damage: true,
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
