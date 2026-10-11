//! Energy Retrieval (BS 81): trade 1 of the other cards in your hand for up to 2 basic Energy cards from your discard pile.
//!
//! Usable with a Basic Energy card in the discard pile and another card in the hand (usability by the rules: a Trainer
//! isn't played for no effect, id925). The hand card is discarded (a Discard event), then up to 2 basic Energy cards (at
//! least 1; at most the count before the discard) go from the discard pile into the hand (PutIntoHand).
use crate::spec::prelude::*;

const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);
const DISCARD: ZoneRef = ZoneRef(Who::Me, Zone::Discard);

pub static SPEC: CardSpec = CardSpec {
    class: "EnergyRetrieval@BS",
    // Trade 1 of the other cards in your hand for up to 2 Basic Energy cards from your discard pile.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            // The Basic Energy available is counted before the discard.
            Step::new(Op::Snapshot(SnapshotSpec { zone: DISCARD, predicate: Pred::BasicEnergy, into: 1 })),
            Step::new(Op::Pick(PickSpec { from: HAND, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Discard(DiscardSpec { from: HAND, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec {
                from: DISCARD,
                predicate: Pred::BasicEnergy,
                bounds: Bounds { min: Num::Lit(1), max: Num::Min(&Num::Lit(2), &Num::RegCount(1)) },
                into: 0,
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: DISCARD, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
