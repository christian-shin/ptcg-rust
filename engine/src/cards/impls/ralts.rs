//! Ralts (M1S / ASC): Collect - draw a card. Headbutt - 10.
//!
//! Twinleaf: MOVE_CARDS from the deck to the hand with `count: 1` and
//! `sourceCard` this.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ralts@MEG|ASC",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
