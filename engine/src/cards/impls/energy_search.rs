//! Energy Search (SVI): search your deck for a Basic Energy card, reveal it,
//! and put it into your hand. Then, shuffle your deck.
//!
//! Twinleaf order: MOVE_CARDS (even with nothing chosen), then the reveal,
//! then the ShuffleDeckPrompt.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EnergySearch",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
