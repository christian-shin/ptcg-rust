//! Hitmontop (ASC 102): Spin and Draw — shuffle your hand into your deck. Then, draw 6 cards. Low Kick — 50.
//!
//! `Op::HandShuffleDraw`: the hand goes into the deck, the deck is shuffled, then one Draw event of 6 cards (as Lacey PRE 114).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HitmontopASC",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(6) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
