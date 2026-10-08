//! Brilliant Blender (SSP, ACE SPEC): search your deck for up to 5 cards and
//! discard them. Then, shuffle your deck.
//!
//! Twinleaf quirks kept: the choice is min 1 / max 5; the chosen cards are
//! shown to the opponent before they are discarded; the final shuffle has
//! no trailing wait.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BrilliantBlender",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(5) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Discard { reveal: true },
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
