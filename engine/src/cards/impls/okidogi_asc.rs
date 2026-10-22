//! Okidogi (ASC 122): Light Punch — 20. Settle the Score — 80+; this attack does 60 more damage for each Prize card your opponent
//! took during their last turn.
//!
//! A before-damage `DamageOp::Add` of 60 times the Prize cards the opponent took during their own last turn
//! (`Num::PrizesTakenLastTurn`: Prize cards taken between turns or during your own turn aren't counted).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "OkidogiASC",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::PrizesTakenLastTurn(Who::Opp), &Num::Lit(60)), when: Cond::True }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
