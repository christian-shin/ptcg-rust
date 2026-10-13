//! Judge (POR 76; also FST, SVI): each player shuffles their hand into their deck and draws 4 cards.
//!
//! Each player's hand (this card is already in play, so not in it) is shuffled into their deck (`HandShuffleDraw`) and they draw
//! 4: yours first, then the opponent's. Usable when you or your opponent has a card in hand or deck (JP FAQ Judge: 「自分
//! または相手の手札か山札が1枚以上あれば、サポート「ジャッジマン」を使うことができます」; batch 8 C12c: it looked at your side only).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Judge@FST|POR",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Any(&[Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any), Cond::Nonempty(ZoneRef(Who::Opp, Zone::Deck), Pred::Any)])],
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(4) })),
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(4) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
