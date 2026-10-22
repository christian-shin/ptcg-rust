//! Klink (SCR 99): Allure - draw a card. Beam - 20.
//!
//! One Draw event (cause: this attack; Electrike PBL 23).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KlinkSCR",
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
