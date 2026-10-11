//! Energy Retrieval (CRI 108): Item. Put up to 2 Basic Energy cards from your discard pile into your hand.
//!
//! An Item's "up to 2" is 1 or 2 (id2301). The chosen cards are shown to the opponent and put into the hand by one
//! PutIntoHand event from the discard pile (cause: this Item). It can't be played without a Basic Energy card in the
//! discard pile (a Trainer that can't have any effect can't be played, id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EnergyRetrieval@CRI",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(1), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy), &Num::Lit(2)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
