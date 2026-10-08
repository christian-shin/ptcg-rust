//! Torchic (DRI): Collect — draw a card. Combustion — 10.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Torchic",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
