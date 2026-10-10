//! Ciphermaniac's Codebreaking (TEF): search your deck for 2 cards, shuffle
//! your deck, then put those cards on top of it in any order.
//!
//! Fixed (phase 4b): the search is for min(2, deck size) cards (min 2 was
//! unanswerable with a single card in the deck).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "CiphermaniacsCodebreaking",
    // Search your deck for 2 cards, shuffle your deck, then put those cards on top of it in any
    // order (2 cards, or the whole deck when it is smaller).
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: DECK,
                bounds: Bounds { min: Num::Min(&Num::Lit(2), &Num::ZoneSize(DECK)), max: Num::Min(&Num::Lit(2), &Num::ZoneSize(DECK)) },
                into: 0,
                msg: "CHOOSE_CARDS",
                ..PickSpec::DEFAULT
            })),
            // The found cards are set aside while the deck is shuffled, then put on top in the order chosen.
            Step::new(Op::Look(LookSpec { from: DECK, cards: CardSel::Chosen(0), into: 1, ..LookSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(1)), cards: CardSel::All, position: DeckPosition::Top, order: DeckOrder::ChosenBy(Who::Me), ..PutIntoDeckSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
