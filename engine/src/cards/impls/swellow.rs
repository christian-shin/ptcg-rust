//! Swellow (DRI 157): Add On - draw 3 cards. Speed Wing - 70.
//!
//! Add On is one Draw event of 3 (cause: this attack), after the damage (this attack does none; Binacle POR 42, Bisharp ASC
//! 147).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Swellow",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
