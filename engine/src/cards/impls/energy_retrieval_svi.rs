//! Energy Retrieval (SVI, as Energy Retrieval FA CRI): put 2 basic Energy
//! cards from your discard pile into your hand.
//!
//! Twinleaf (scarlet-and-violet file): a DiscardToHandEffect probe first (if
//! prevented, nothing happens); throws with no Basic Energy in the discard;
//! ChooseCardsPrompt min 1, max min(2, Basic Energy), no cancel. The callback
//! moves the cards, shows them to the opponent (ShowCardsPrompt), then runs
//! the same MOVE_CARDS again (the cards are no longer in the discard).
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
                shuffle_first: false,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
