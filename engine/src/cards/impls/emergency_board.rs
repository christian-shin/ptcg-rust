//! Rescue Board (PRE 126 / TEF 159, Pokémon Tool): the Retreat Cost of the Pokémon this card is attached to is [C] less. If that
//! Pokémon's remaining HP is 30 or less, it has no Retreat Cost.
//!
//! Two `Modifier::RetreatCost` declarations on the Holder: `Reduce(1)` while its remaining HP (maximum HP after effects
//! minus its damage) is above 30, and `Free` ("has no Retreat Cost") at 30 or less. The cost is one computation (APR D-11 /
//! D-12 / D-13: a "has no" wins over a "less"); a Tool lock turns both off (JP FAQ Rescue Board: [C] less on a full-HP Basic
//! with a [C] Retreat Cost leaves none).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RescueBoard",
    passives: &[
        // [C] less; with 30 or less HP remaining, no Retreat Cost.
        Passive {
            origin: RuleSource::Tool,
            modifier: Modifier::RetreatCost(RetreatCostSpec {
                change: CostChange::Free,
                subject: SlotPred::All(&[SlotPred::Holder, SlotPred::RemainingHpAtMost(30)]),
                ..RetreatCostSpec::DEFAULT
            }),
        },
        Passive {
            origin: RuleSource::Tool,
            modifier: Modifier::RetreatCost(RetreatCostSpec {
                change: CostChange::Reduce(Num::Lit(1)),
                subject: SlotPred::All(&[SlotPred::Holder, SlotPred::Not(&SlotPred::RemainingHpAtMost(30))]),
                ..RetreatCostSpec::DEFAULT
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
