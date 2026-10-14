//! Cynthia's Gabite (DRI): Champion's Call - once during your turn, search your deck for a Cynthia's Pokémon, reveal it, put
//! it into your hand. Then, shuffle your deck. Dragon Slice - 40.
//!
//! One Search op revealing what it finds (min 0: a hidden zone may be searched without finding), then one shuffle after it
//! (APR E-19). The Ability needs no card in the deck to be used.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasGabite",
    // Champion's Call: once during your turn, search your deck for a Cynthia's Pokémon, reveal it,
    // put it into your hand, then shuffle.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::CYNTHIAS)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
