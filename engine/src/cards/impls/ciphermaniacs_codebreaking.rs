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
            Step::new(Op::Move(MoveSpec { from: DECK, to: ZoneRef(Who::Me, Zone::Scratch(1)), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK })),
            Step::new(Op::Order(OrderSpec { reg: 1 })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(1)), to: DECK, cards: CardSel::All, place: Place::Top, ..MoveSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
