//! Boxed Order (TEF): search your deck for up to 2 Item cards, reveal them,
//! and put them into your hand. Then, shuffle your deck. Your turn ends.
//!
//! Fixed (A-PC3): the turn ends after the search, reveal and shuffle, in the
//! printed order (Twinleaf ended it before the search was answered).
//! Fixed (phase 4b): the deck is shuffled after the reveal (the shuffle was
//! only reached when no card was chosen). Fixed (R3): min is 0 (it was 1 when
//! the deck held an Item); a search may find fewer cards, even none
//! (Rulings Compendium 780), and it is unplayable with an empty deck (779).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "DeliveryBox",
    // Search your deck for up to 2 Item cards, reveal them, and put them into your hand. Then,
    // shuffle your deck. Your turn ends.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::Item, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            Step::new(Op::EndTurn(EndTurnSpec { who: Who::Me })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
