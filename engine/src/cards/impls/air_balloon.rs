//! Air Balloon (SSH, tool): the Retreat Cost of the Pokémon this card is
//! attached to is [C][C] less.
//!
//! The reduction is added to `CheckRetreatCostEffect.costReduction` and applied by the core
//! after every handler ran, together with the increases (Advanced Rulebook D-11, D-12).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AirBalloon",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::RetreatCost(RetreatCostSpec { change: CostChange::Reduce(Num::Lit(2)), side: Side::Owner, ..RetreatCostSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
