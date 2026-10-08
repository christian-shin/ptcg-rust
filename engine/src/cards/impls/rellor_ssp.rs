//! Rellor (SSP): Collect — draw a card. Rollout — 10.
//!
//! Twinleaf: returns early with an empty deck, otherwise MOVE_CARDS
//! (count 1, sourceCard) from deck to hand.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rellor@SSP|Rellor DRI",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
