//! Cassiopeia (SFA): only when it is the last card in your hand; search your
//! deck for up to 2 cards, put them into your hand, shuffle.
//!
//! Twinleaf: no Supporter-already-played check in the card (the core rejects
//! it), no move to the supporter pile, no preventDefault, no reveal; the
//! prompt takes 1-2 cards (R5 had made it 0-2; phase 4b, rulings 325/892/1778/1792:
//! a search for "any card" of an unspecified kind in a deck known to hold cards
//! must take at least 1) and the shuffle prompt has no wait.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Cassiopeia",
    // Only when it is the last card in your hand: search your deck for up to 2 cards (at least 1
    // from a deck known to hold cards), put them into your hand, shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::LastCardInHand(Who::Me)],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
