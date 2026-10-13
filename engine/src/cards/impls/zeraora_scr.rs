//! Zeraora (SCR 55): Combat Thunder — 20+; this attack does 20 more damage for each of your opponent's Benched Pokémon.
//!
//! An `Add` of 20 per opposing Benched Pokémon to the printed 20, before Weakness and Resistance.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zeraora@SCR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::BenchCount(Who::Opp), &Num::Lit(20)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
