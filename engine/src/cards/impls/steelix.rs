//! Steelix (M1L / MEG 93): Welcoming Tail — 40+; 200 more if you have exactly
//! 6 Prize cards remaining. Skull Bash — 140.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Steelix",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(200), when: Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Eq, Num::Lit(6)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
