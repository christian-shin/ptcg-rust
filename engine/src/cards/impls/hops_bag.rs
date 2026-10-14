//! Hop's Bag (JTG 147): search your deck for up to 2 Basic Hop's Pokémon and put them onto your Bench. Then, shuffle your
//! deck.
//!
//! The chosen Pokémon enter play from the deck (EnterPlay events, this Item their cause); the deck is shuffled after the
//! search whatever was found (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HopsBag",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Basic, Pred::Tag(crate::types::tag::HOPS)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
