//! Lillie's Determination (M1L): shuffle your hand into your deck, then draw
//! 6 cards (8 if you have exactly 6 Prize cards remaining).
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
