//! Energy Search Pro (SSP, ACE SPEC): search your deck for any number of
//! Basic Energy cards of different types, reveal them, and put them into
//! your hand. Then, shuffle your deck.
//!
//! Twinleaf: no preventDefault (the item is discarded right away); max =
//! number of distinct `provides[0]` among the deck's Basic Energy. In the
//! prompt callback: same-name check on the first two cards (throws),
//! SHOW_CARDS_TO_PLAYER, MOVE_CARDS, then (fixed in phase 4b: the callback
//! never called `next()`) the generator continues and shuffles the deck.
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
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
