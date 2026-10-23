//! Carnivine (CRI 4): Chomp Whole - 80+; if your opponent's Active Pokémon has no Retreat Cost, this attack does 80 more damage.
//!
//! The Retreat Cost is read as the game checks it (effects that raise or remove it count): 0 [C] adds 80 before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Carnivine",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(80, Cond::Cmp(Num::RetreatCostColorless(Who::Opp), CmpOp::Eq, Num::Lit(0))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
