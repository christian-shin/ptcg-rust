//! Air Balloon (SSH, tool): the Retreat Cost of the Pokémon this card is attached to is [C][C] less.
//!
//! A `RetreatCost` reduction over the Holder: the "more" and "less" changes are summed and floored at zero in one computation
//! (APR D-11, D-12).
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
