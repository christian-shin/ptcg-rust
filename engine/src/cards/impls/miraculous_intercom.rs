//! Miracle Headset (SSP 183): Item. Put up to 2 Supporter cards from your discard pile into your hand.
//!
//! An Item's "up to 2" is 1 or 2 (id2301). The chosen cards are put into the hand (revealed by the move, id787) by one
//! PutIntoHand event from the discard pile (cause: this Item). It can't be played without a Supporter in the discard pile
//! (a Trainer that can't have any effect can't be played, id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiracleHeadset",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                cancel: false,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
