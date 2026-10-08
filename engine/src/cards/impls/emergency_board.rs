//! Rescue Board (TEF, Pokémon Tool): the Retreat Cost of the Pokémon this
//! card is attached to is [C] less. If that Pokémon's remaining HP is 30 or
//! less, it has no Retreat Cost.
//!
//! Twinleaf: the tool block probe comes after the top-card lookup.
//!
//! Fixed (phase 4b, R2): the remaining HP was the printed HP minus the
//! Active's damage; it is now the current HP (CheckHpEffect: Stadium, Ability
//! and Energy bonuses) minus the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EmergencyBoard",
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
