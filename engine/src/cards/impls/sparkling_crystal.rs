//! Sparkling Crystal (PRE 129 / SCR 142, ACE SPEC Pokémon Tool): when the Tera Pokémon this card is attached to uses an attack,
//! that attack costs 1 Energy less. (The Energy can be of any type.)
//!
//! An `AttackCost` modifier (`CostChange::AnyOne`) whose subject is the Holder while it is a Tera Pokémon: one symbol of the cost,
//! of any type, is removed and summed with the other cost changes (D-11 / D-12), so it works down to a cost of 0.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "SparklingCrystal",
    // The attacks of the Tera Pokémon this card is attached to cost 1 Energy less.
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::AnyOne,
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::Top(Pred::Tag(tag::POKEMON_TERA))]),
            ..AttackCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
