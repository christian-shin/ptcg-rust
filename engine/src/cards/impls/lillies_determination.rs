//! Lillie's Determination (ASC 192): Supporter. Shuffle your hand into your deck. Then, draw 6 cards. If you have exactly 6
//! Prize cards remaining, draw 8 cards instead.
//!
//! The hand goes into the deck by one PutIntoDeck event (cause: this Supporter), the deck is shuffled once, then one Draw
//! event of 6 cards (8 with exactly 6 Prize cards left, read when the draw starts).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LilliesDetermination",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec {
            who: Who::Me,
            draw: Num::If(&Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Eq, Num::Lit(6)), &Num::Lit(8), &Num::Lit(6)),
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
