//! Counter Gain (LOT / ASC, tool): if you have more Prize cards remaining than your opponent, the attacks of the Pokémon this
//! card is attached to cost [C] less.
//!
//! An `AttackCost` reduction of 1 over the Holder's attacks, guarded by the owner having more Prize cards left than the
//! opponent; a Tool-effects lock turns it off. Summed with the other changes and floored at zero (APR D-11, D-12).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CounterGain",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::Reduce(Num::Lit(1)),
            // More Prize cards remaining than your opponent.
            guard: Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Gt, Num::PrizesLeft(Who::Opp)),
            ..AttackCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
