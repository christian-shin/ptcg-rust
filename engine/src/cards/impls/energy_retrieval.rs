//! Energy Retrieval (BS): trade 1 of the other cards in your hand for
//! up to 2 basic Energy cards from your discard pile.
//!
//! Twinleaf: throws with no Basic Energy in the discard or no other card in
//! hand (phase 4b fix; the prompt was unanswerable); the hand card is
//! chosen from a temporary copy of the hand (min 1, no cancel); the second
//! prompt's max is min(2, Basic Energy counted before the discard), min 1.
use crate::spec::prelude::*;

const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);
const DISCARD: ZoneRef = ZoneRef(Who::Me, Zone::Discard);

pub static SPEC: CardSpec = CardSpec {
    class: "EnergyRetrieval@BS",
    // Trade 1 of the other cards in your hand for up to 2 Basic Energy cards from your discard pile.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Nonempty(DISCARD, Pred::BasicEnergy), Cond::NonemptyOther(HAND, Pred::Any)],
        steps: &[
            // The Basic Energy available is counted before the discard.
            Step::new(Op::Snapshot(SnapshotSpec { zone: DISCARD, predicate: Pred::BasicEnergy, into: 1 })),
            Step::new(Op::Pick(PickSpec { from: HAND, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: HAND, to: DISCARD, cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec {
                from: DISCARD,
                predicate: Pred::BasicEnergy,
                bounds: Bounds { min: Num::Lit(1), max: Num::Min(&Num::Lit(2), &Num::RegCount(1)) },
                into: 0,
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Move(MoveSpec { from: DISCARD, to: HAND, cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
