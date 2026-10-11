//! Lacey (PRE): shuffle your hand into your deck. Then, draw 4 cards. If your opponent has 3 or fewer Prize cards remaining,
//! draw 8 cards instead.
//!
//! `Op::HandShuffleDraw`: the hand (without Lacey) goes into the deck, the deck is shuffled, then one Draw event of 4 cards
//! (8 when the opponent has 3 or fewer Prize cards left).
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
