//! Max Rod (PRE, ACE SPEC Item): put up to 5 in any combination of Pokémon and Basic Energy cards from your discard pile into
//! your hand.
//!
//! A PutIntoHand of the Item's effect from the discard pile. The discard pile is known, so a Trainer's "up to"
//! takes at least 1 (id2301); with none of those cards there the card has no effect (id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MaxRod",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::OneOf(&[Pred::Pokemon, Pred::BasicEnergy]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(5) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                cancel: false,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
