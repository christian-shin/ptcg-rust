//! Throh (BLK 50): Shoulder Throw - 120-; this attack does 30 less damage for each [C] in your opponent's Active Pokémon's Retreat
//! Cost.
//!
//! The damage is 120 minus 30 for each [C] of the Defending Pokémon's Retreat Cost as the game checks it, never below 0, set before
//! the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Throh",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Max(&Num::Lit(0), &Num::Sub(&Num::Lit(120), &Num::Mul(&Num::RetreatCostColorless(Who::Opp), &Num::Lit(30))))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
