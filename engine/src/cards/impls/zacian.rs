//! Zacian (M2 / PFL): Limit Break — 50; if your opponent has 3 or fewer Prize
//! cards remaining, 90 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zacian@PFL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(90), when: Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(3)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
