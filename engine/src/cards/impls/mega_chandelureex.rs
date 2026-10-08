//! Mega Chandelure ex (PBL / M5): Binding Flame - your opponent's Active
//! Pokémon's Retreat Cost is [C] more. Phantom Maze - 130+; 50 more damage for
//! each [C] in your opponent's Active Pokémon's Retreat Cost.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaChandelureex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Add(&Num::Lit(130), &Num::Mul(&Num::RetreatCostColorless(Who::Opp), &Num::Lit(50)))))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::RetreatCost(RetreatCostSpec { change: CostChange::Add(1), side: Side::Opponent, subject: SlotPred::Any, ..RetreatCostSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
