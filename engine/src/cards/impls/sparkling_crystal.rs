//! Sparkling Crystal (SCR, ACE SPEC Pokémon Tool): when the Tera Pokémon this
//! card is attached to uses an attack, that attack costs 1 Energy less.
//!
//! Twinleaf: on a CheckAttackCostEffect while the tool is on the player's
//! Active: a ToolEffect stub (not IS_TOOL_BLOCKED) must not throw, the Active
//! must be Tera; the provided Energy units (CheckProvidedEnergyEffect on the
//! Active) pay each printed cost slot in order ([C] with any unit, a typed
//! slot with a matching unit, else with a rainbow unit); if the covered slots
//! number at least cost length - 1 the cost becomes the covered units.
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
