//! Yungoos (M1L): Collect — draw a card.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Yungoos",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
