//! Colress's Tenacity (SFA 57): Supporter. Search your deck for a Stadium card and an Energy card, reveal them, and put
//! them into your hand. Then, shuffle your deck.
//!
//! At most one Stadium and one Energy card (the caps); the cards are revealed and put into the hand (a PutIntoHand event),
//! then the deck is shuffled once (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "ColresssObsession",
    // Search your deck for a Stadium card and an Energy card, reveal them, put them into your
    // hand, shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    predicate: Pred::OneOf(&[Pred::Stadium, Pred::Energy]),
                    bounds: Bounds {
                        min: Num::Lit(0),
                        max: Num::Add(&Num::Min(&Num::CardCount(DECK, Pred::Stadium), &Num::Lit(1)), &Num::Min(&Num::CardCount(DECK, Pred::Energy), &Num::Lit(1))),
                    },
                    caps: &[
                        Cap { kind: CapKind::Trainer, max: Num::Min(&Num::CardCount(DECK, Pred::Stadium), &Num::Lit(1)) },
                        Cap { kind: CapKind::Energy, max: Num::Min(&Num::CardCount(DECK, Pred::Energy), &Num::Lit(1)) },
                    ],
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
