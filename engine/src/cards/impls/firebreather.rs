//! Firebreather (M2 / PFL): search your deck for up to 7 Basic [R] Energy cards, reveal them, put them into your hand. Then,
//! shuffle your deck.
//!
//! One Search op over Basic Fire Energy cards, revealed, then one shuffle after it (APR E-19, id2303).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Firebreather",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(7) }, ..PickSpec::DEFAULT },
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
