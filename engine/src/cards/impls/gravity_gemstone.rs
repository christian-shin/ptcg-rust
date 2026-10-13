//! Gravity Gemstone (SCR 137, Pokémon Tool): as long as the Pokémon this card is attached to is in the Active Spot, the Retreat
//! Cost of both Active Pokémon is [C] more.
//!
//! One `Modifier::RetreatCost` (`Add(1)`, either Active Pokémon, the Holder Active). The cost is one computation (APR D-11 /
//! D-12 / D-13): the "more" and "less" are summed and floored at zero, and a "has no Retreat Cost" (Skyliner, Metal Bridge)
//! wins; a Tool lock turns it off.
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
