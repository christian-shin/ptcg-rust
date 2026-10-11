//! Energy Search Pro (SSP 176, ACE SPEC): search your deck for any number of Basic Energy cards of different types, reveal
//! them, and put them into your hand. Then, shuffle your deck.
//!
//! The chosen cards (at most one per type) are revealed and go from the deck into the hand (PutIntoHand, this Item its
//! cause); the deck is shuffled after the search (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "EnergySearchPro",
    // Search your deck for any number of Basic Energy cards of different types, reveal them, and
    // put them into your hand. Then, shuffle your deck.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    predicate: Pred::BasicEnergy,
                    bounds: Bounds { min: Num::Lit(0), max: Num::DistinctTypes(DECK, Pred::BasicEnergy) },
                    distinct_types: true,
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
