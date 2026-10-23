//! Treasure Tracker (PRE 131, Item, ACE SPEC): search your deck for up to 5 Pokémon Tool cards, reveal them, and put them into
//! your hand. Then, shuffle your deck.
//!
//! One search of 0 to 5 Pokémon Tool cards revealed into the hand (Boxed Order's form), the shuffle after it (APR E-19). A deck
//! with cards in it and no Tool is still searched and the Item played (id929); an empty deck can't be searched (id840).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "TreasureTracker",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: DECK, predicate: Pred::Tool, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(5) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
