//! Hop's Bag (JTG): search your deck for up to 2 Basic Hop's Pokémon and put
//! them onto your Bench. Then, shuffle your deck.
//!
//! Twinleaf: deck cards without the Hop's tag are blocked (filter: Basic
//! Pokémon); each chosen card is played with a PlayPokemonFromDeckEffect into
//! the empty Bench slots taken before the prompt; the card then moves
//! supporter→discard before a wait-less shuffle.
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
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
