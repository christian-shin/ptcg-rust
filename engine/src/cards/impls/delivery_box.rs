//! Boxed Order (TEF 143): search your deck for up to 2 Item cards, reveal them, and put them into your hand. Then, shuffle
//! your deck. Your turn ends.
//!
//! One Search op revealing what it finds (min 0: it may find fewer, even none), one shuffle after it, then the turn ends: the
//! end of the turn is the last step (id1963: it ends the turn even when no Item was found).
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
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            Step::new(Op::EndTurn(EndTurnSpec { who: Who::Me })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
