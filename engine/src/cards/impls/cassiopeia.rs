//! Cassiopeia (SFA): you can use this card only when it is the last card in your hand (printed: `Cond::Printed`). Search your
//! deck for up to 2 cards and put them into your hand. Then, shuffle your deck.
//!
//! Searching for "a card" of any kind in a deck known to hold cards takes at least 1 (id2301, id347, id968; JP FAQ
//! Cassiopeia); the shuffle always follows the search (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Cassiopeia",
    // Only when it is the last card in your hand: search your deck for up to 2 cards (at least 1
    // from a deck known to hold cards), put them into your hand, shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::LastCardInHand(Who::Me))],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
