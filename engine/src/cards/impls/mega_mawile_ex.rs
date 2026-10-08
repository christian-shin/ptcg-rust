//! Mega Mawile ex (M1L / MEG 94): Gobble Down — 80x for each Prize card you
//! have taken. Huge Bite — 260; if the opponent's Active already has damage
//! counters, the base damage is 30.
//!
//! Twinleaf: both attacks overwrite `effect.damage`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaMawileEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::Sub(&Num::Lit(6), &Num::PrizesLeft(Who::Me)), &Num::Lit(80)), when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Lit(30), when: Cond::Slot(OPP_ACTIVE, SlotPred::Damaged) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
