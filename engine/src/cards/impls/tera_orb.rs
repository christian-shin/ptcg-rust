//! Tera Orb (SSP 189): Item. Search your deck for a Tera Pokémon, reveal it, and put it into your hand. Then, shuffle your
//! deck.
//!
//! The card is revealed and put into the hand (a PutIntoHand event, cause: this Item), then the deck is shuffled once
//! (APR E-19); a Tera Pokémon may fail to be found.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeraOrb",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::POKEMON_TERA)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
