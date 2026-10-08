//! Counter Gain (LOT / ASC, tool): if you have more Prize cards remaining
//! than your opponent, the attacks of the Pokémon this card is attached to
//! cost [C] less.
//!
//! Twinleaf: the tool must be on the attacker's Active; IS_TOOL_BLOCKED must
//! not hold (phase 4b: it used to be a bare ToolEffect stub that ignored the
//! "Stadiums and Tools have no effect" turns); then the first [C] of the cost
//! is removed (the card says [C] less) when the attacker has more Prize cards
//! left.
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
