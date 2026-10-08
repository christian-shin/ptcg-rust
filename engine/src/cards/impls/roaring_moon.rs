//! Roaring Moon (TEF): Vengeance Fletching — 70+, 10 more for each Ancient
//! card in your discard pile. Speed Wing — 120.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RoaringMoon",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec {
            op: DamageOp::Add,
            hp: Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Tag(crate::types::tag::ANCIENT)), &Num::Lit(10)),
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
