//! Rowlet (SFA): Add On — draw a card. Leafage — 10.
//!
//! Twinleaf: MOVE_CARDS (count 1, sourceCard) from deck to hand, with no
//! empty-deck check.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rowlet@SFA",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
