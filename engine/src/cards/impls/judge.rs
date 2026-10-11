//! Judge (POR 76; also FST, SVI): each player shuffles their hand into their deck and draws 4 cards.
//!
//! Each player's hand (this card is already in play, so not in it) is shuffled into their deck (`HandShuffleDraw`) and they draw
//! 4: yours first, then the opponent's. Usable when either you have another card in hand or the deck is not empty.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Judge@FST|POR",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Any(&[Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)])],
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(4) })),
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(4) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
