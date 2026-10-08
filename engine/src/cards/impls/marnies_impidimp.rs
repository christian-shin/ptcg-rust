//! Marnie's Impidimp (DRI): Filch — draw a card. Corkscrew Punch — 10.
//!
//! Twinleaf draws in AFTER_ATTACK (after damage).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarniesImpidimp",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
