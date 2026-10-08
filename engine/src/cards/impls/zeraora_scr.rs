//! Zeraora (SCR): Combat Thunder — 20+, 20 more for each of your opponent's
//! Benched Pokémon.
//!
//! Twinleaf (stellar-crown file): sets `effect.damage = 20 + 20 × occupied
//! opponent bench slots` (overwrites earlier modifications).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zeraora@SCR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Add(&Num::Lit(20), &Num::Mul(&Num::BenchCount(Who::Opp), &Num::Lit(20))), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
