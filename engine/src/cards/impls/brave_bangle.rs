//! Brave Bangle (SV11W, tool): if the holder doesn't have a Rule Box, its attacks do 30 more damage to the opponent's Active
//! Pokémon ex (before Weakness and Resistance).
//!
//! A `DamageDealt` bonus at the Deal stage: the attacker is the Holder without a Rule Box, the damaged Pokémon is the
//! opponent's Active Pokémon ex, and the attack's printed damage is above 0 (so it never applies to counters, JP FAQ Brave
//! Bangle).
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
