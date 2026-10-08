//! Hop's Cramorant (JTG): Fickle Spitting — 120; if your opponent doesn't
//! have exactly 3 or 4 Prize cards remaining, this attack does nothing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HopsCramorant",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Lit(0), when: Cond::Not(&Cond::Any(&[Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Eq, Num::Lit(3)), Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Eq, Num::Lit(4))])) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
