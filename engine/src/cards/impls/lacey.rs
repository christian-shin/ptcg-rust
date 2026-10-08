//! Lacey (SCR, supporter): shuffle your hand into your deck, then draw 4
//! cards (8 if your opponent has 3 or fewer Prize cards remaining).
//!
//! Twinleaf: SHUFFLE_HAND_INTO_DECK_THEN_DRAW excluding this card; the draw
//! count uses `getPrizeLeft() > 3`.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Lacey",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::If(&Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Gt, Num::Lit(3)), &Num::Lit(4), &Num::Lit(8)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
