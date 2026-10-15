//! Mega Chandelure ex (PBL / M5): Binding Flame - your opponent's Active
//! Pokémon's Retreat Cost is [C] more. Phantom Maze - 130+; 50 more damage for
//! each [C] in your opponent's Active Pokémon's Retreat Cost.
//!
//! Binding Flame is a `RetreatCost` `Add(1)` on the opponent's Active Pokémon while the Ability works, summed with the other changes
//! (D-11 / D-12); a "no Retreat Cost" such as Latias ex's Skyliner wins over it (D-13, JP FAQ Mega Chandelure ex). Phantom Maze counts
//! the [C] in the opponent's Active Pokémon's Retreat Cost as it is now (none under Skyliner).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaChandelureEx",
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
