//! Incineroar ex (TEF): Hustle Play — attacks used by this Pokémon cost [C]
//! less for each of your opponent's Benched Pokémon. Blaze Blast — 240; the
//! opponent's Active Pokémon is now Burned.
//!
//! Twinleaf: the CheckAttackCostEffect handler has no attack check (any
//! attack cost check while this is the Active) and does
//! `cost.splice(cost.indexOf(C), benched)`; with no [C] in the cost,
//! `indexOf` is -1 and `splice(-1, n)` removes the last element.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Incineroarex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(inflict(&[SpecialCondition::Burned])),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::AttackCost(AttackCostSpec { change: CostChange::Reduce(Num::BenchCount(Who::Opp)), attack: None, subject: SlotPred::Holder, ..AttackCostSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
