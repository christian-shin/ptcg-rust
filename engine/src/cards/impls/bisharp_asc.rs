//! Bisharp (ASC 147): Rapid Draw - 50; draw 2 cards.
//!
//! A Draw of 2 after the damage (Binacle POR 42).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BisharpASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
