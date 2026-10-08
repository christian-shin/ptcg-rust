//! Binacle (M3 / POR 42): Double Draw — draw 2 cards (on AfterAttackEffect).
//! Scratch — 30.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Binacle",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
