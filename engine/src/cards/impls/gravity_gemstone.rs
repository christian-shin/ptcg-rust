//! Gravity Gemstone (SCR, tool): as long as the Pokémon this card is
//! attached to is in the Active Spot, the Retreat Cost of both Active
//! Pokémon is [C] more.
//!
//! Twinleaf: on a CheckRetreatCostEffect, unless the tool is blocked for the
//! effect's player, a [C] is pushed when either Active holds this tool. A cost that
//! an effect set to none (Skyliner, Metal Bridge; ruling 1617) is emptied by the core
//! afterwards (`no_cost`); a cost reduced to 0 by "less" effects is calculated
//! together with the increase (Advanced Rulebook D-11, D-12; ruling 836).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GravityGemstone",
    passives: &[Passive {
        origin: RuleSource::Tool,
        // Both Active Pokémon, while the holder is Active.
        modifier: Modifier::RetreatCost(RetreatCostSpec {
            change: CostChange::Add(1),
            which: RetreatWhich::Either,
            subject: SlotPred::Holder,
            ..RetreatCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
