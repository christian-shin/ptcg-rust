//! Tapu Koko ex (JTG 51): Linked Lightning — 60+; this attack does 20 more damage for each of your Benched Pokémon.
//!
//! An `Add` of 20 per Benched Pokémon of yours to the printed 60, before Weakness and Resistance.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TapuKokoEx",
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
