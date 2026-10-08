//! Cynthia's Gabite (DRI): Champion's Call - once during your turn, search
//! your deck for a Cynthia's Pokémon, reveal it, put it into your hand, then
//! shuffle. Dragon Slice - 40.
//!
//! Twinleaf: ABILITY_USED runs before the prompt; the choice is min 0 / max 1
//! with no cancel (non-Cynthia's Pokémon blocked, works on an empty deck);
//! the reveal to the opponent and the ShuffleDeckPrompt are queued together;
//! the once-per-turn marker is only added in the shuffle callback.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasGabite",
    // Champion's Call: once during your turn, search your deck for a Cynthia's Pokémon, reveal it,
    // put it into your hand, then shuffle.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("CHAMPIONS_CALL_MARKER"),
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::CYNTHIAS)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
