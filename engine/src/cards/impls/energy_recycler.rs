//! Energy Recycler (BST 124): shuffle up to 5 basic Energy cards from your discard pile into your deck.
//!
//! The chosen cards (at least 1 when the discard pile has any) go from the discard pile into the deck (PutIntoDeck), then
//! the deck is shuffled (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EnergyRecycler",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(5) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Deck { reveal: false },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
