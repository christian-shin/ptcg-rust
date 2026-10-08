//! Ethan's Typhlosion (DRI): Buddy Blast — 40+; 60 more damage for each
//! Ethan's Adventure card in your discard pile. Steam Artillery — 160.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansTyphlosion",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Name("Ethan's Adventure")), &Num::Lit(60)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
