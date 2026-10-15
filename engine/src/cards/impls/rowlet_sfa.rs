//! Rowlet (SFA 3): Add On: draw a card. Leafage: 10.
//!
//! The draw is a Draw event (this attack its cause); with an empty deck nothing is drawn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RowletSFA",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
