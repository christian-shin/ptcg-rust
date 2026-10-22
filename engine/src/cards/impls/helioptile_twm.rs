//! Helioptile (TWM 70): Collect - draw a card. Static Shock - 10.
//!
//! One Draw event (cause: this attack; Electrike PBL 23).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HelioptileTWM",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
