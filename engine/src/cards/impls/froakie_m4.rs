//! Froakie (CRI / M4): Collect — draw a card. Water Gun — 10.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Froakie@CRI",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
