//! Iron Crown (SSP 132): Deleting Slash - 40+; if your opponent has 3 or more Benched Pokémon, this attack does 80 more
//! damage. Slicing Blade - 100.
//!
//! Read before the damage: the opponent's Benched Pokémon, 3 or more.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "IronCrownSSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(80, Cond::Cmp(Num::BenchCount(Who::Opp), CmpOp::Ge, Num::Lit(3)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
