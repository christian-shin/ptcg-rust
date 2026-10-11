//! Prism Tower (CRI 80, Stadium): once during each player's turn, that player may discard 2 cards from their hand in order to
//! draw a card.
//!
//! A Stadium use (UseStadium by either player, once per turn each): the 2 discarded cards are a cost (`Step::cost`, APR E-30;
//! id2362: not usable with an empty deck, no discard for no effect), a Discard event over the user's hand; the Draw follows.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PrismTower",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(2), max: Num::Lit(2) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
