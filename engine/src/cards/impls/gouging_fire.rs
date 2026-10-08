//! Gouging Fire (SSP): Knock Down — 30. Blazing Charge — 100+; 70 more if
//! your opponent has 4 or fewer Prize cards remaining.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "GougingFire",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(more_damage_if(70, Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(4))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
