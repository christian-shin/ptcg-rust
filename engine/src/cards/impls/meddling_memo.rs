//! Meddling Memo (SSP 181): your opponent counts the cards in their hand, shuffles those cards, and puts them on the bottom
//! of their deck. If they do, they draw that many cards.
//!
//! The opponent's hand goes to the bottom of their deck in a shuffled order (PutIntoDeck, owner = the opponent); "if they
//! do" is `Cond::Done` on that event (APR E-20; with no cards in their hand it can't be played, id956), and they then draw as
//! many cards as were put (a Draw event).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MeddlingMemo",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::All, position: DeckPosition::Bottom, order: DeckOrder::Shuffled, into: Some(0), ..PutIntoDeckSpec::DEFAULT })),
            // "If they do, they draw that many cards" (E-20: the draw depends on the put).
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::RegCount(0)) }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
