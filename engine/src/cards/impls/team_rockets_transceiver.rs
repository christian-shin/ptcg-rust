//! Team Rocket's Transceiver (DRI, Item): search your deck for a Team Rocket
//! Supporter, reveal it, put it into your hand, then shuffle.
//!
//! Twinleaf: the empty-deck check throws after MOVE_CARDS to the supporter
//! pile; the ShowCards prompt is awaited before the shuffle is created; the
//! shuffle has no animation wait.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsTransceiver",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Supporter, Pred::Tag(crate::types::tag::TEAM_ROCKET)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
