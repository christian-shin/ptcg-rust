//! Drakloak (TWM): Recon Directive - once during your turn, look at the top 2
//! cards of your deck and put 1 of them into your hand; the other goes on the
//! bottom of your deck.
//!
//! Fixed (phase 4b, R3): the choice can't be cancelled (it used to be
//! cancellable, and cancelling made the first MOVE_CARDS move every
//! looked-at card into the hand).
use crate::spec::prelude::*;

const TOP: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(1));

pub static SPEC: CardSpec = CardSpec {
    class: "Drakloak",
    // Recon Directive: once during your turn, look at the top 2 cards of your deck and put 1 of
    // them into your hand; the other goes on the bottom of your deck.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("TELLING_SPIRIT_MARKER"),
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), into: 1, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { from: TOP, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_HAND", ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: TOP, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: TOP, cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
