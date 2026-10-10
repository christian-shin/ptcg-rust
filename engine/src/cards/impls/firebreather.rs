//! Firebreather (M2 / PFL): search your deck for up to 7 Basic [R] Energy
//! cards, reveal them, put them into your hand, shuffle.
//!
//! Twinleaf: the empty-deck check comes before the Supporter-played check;
//! the filter is Basic Energy named "Fire Energy"; ShowCards only when
//! something was taken; the final shuffle prompt has no wait.
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
