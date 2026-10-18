//! Brilliant Blender (SSP 164, ACE SPEC): search your deck for up to 5 cards and discard them. Then, shuffle your deck.
//!
//! A search for any card you want that names no kind takes at least 1 card (id2062, id2301): the pick is min 1 / max 5. The
//! cards found are discarded from the deck, not shown (the text says no "reveal"; id787 reveals only a move from a public zone to a hidden one) (Discard events, one per owner carrying its cards, D3);
//! then one shuffle (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BrilliantBlender",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(5) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Discard { reveal: false },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
