//! Delibird (MEG 105): Quick Gift (usable on the first turn when going first)
//! — search your deck for a card and put it into your hand, then shuffle.
//! Gentle Slap — 30.
//!
//! Twinleaf: SEARCH_DECK_FOR_CARDS_TO_HAND with an empty filter (so the pick is
//! not shown), min 1, max 1, no cancel; nothing happens with an empty deck.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "DelibirdMEGPool",
    // Quick Gift: search your deck for a card and put it into your hand, then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: false },
                    msg: "",
                    cancel: false,
                    shuffle_first: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
