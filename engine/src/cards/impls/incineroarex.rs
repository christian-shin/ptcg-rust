//! Incineroar ex (TEF): Hustle Play - attacks used by this Pokémon cost [C] less for each of your opponent's Benched Pokémon.
//! Blaze Blast - 240; the opponent's Active Pokémon is now Burned.
//!
//! Hustle Play is an `AttackCost` `Reduce` of the number of the opponent's Benched Pokémon, bound to this Pokémon (`Holder`: a
//! copied attack isn't "used by this Pokémon", JP FAQ audit item 1), summed with the other changes and floored at zero (D-11 /
//! D-12). Only [C] is taken off: Blaze Blast's [R] stays (scenarios fix-attack-cost-incineroar-ex-hustle-play-*).
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
