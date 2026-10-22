//! Meditite (ASC 103): Collect - draw a card. Gentle Slap - 10.
//!
//! One Draw event, cause: this attack (Electrike PBL 23's Collect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MedititeASC",
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
