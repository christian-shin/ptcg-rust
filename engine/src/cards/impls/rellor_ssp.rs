//! Rellor (SSP): Collect - draw a card. Rollout - 10.
//!
//! One Draw of 1 card after the damage (a Draw event; with an empty deck nothing is drawn).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rellor@SSP|DRI",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
