//! Team Rocket's Petrel (DRI): search your deck for a Trainer card, reveal it, and put it into your hand. Then, shuffle your
//! deck.
//!
//! One Search op revealing what it finds, then one shuffle after it (APR E-19). Being a Team Rocket's Supporter played from the
//! hand is read from the turn's play record by Team Rocket's Factory (`Cond::PlayedThisTurn`); used through Look-Alike Show it isn't
//! played (D5).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsPetrel",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Trainer, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
