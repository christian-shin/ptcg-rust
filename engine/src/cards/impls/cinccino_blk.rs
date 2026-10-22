//! Cinccino (BLK 76): Do the Wave - 20+; this attack does 20 more damage for each of your Benched Pokémon.
//!
//! Adds 20 per Benched Pokémon of mine before the damage (`Num::BenchCount`, Dipplin TWM 18).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CinccinoBLK",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::BenchCount(Who::Me), &Num::Lit(20)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
