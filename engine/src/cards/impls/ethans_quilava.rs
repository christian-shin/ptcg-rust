//! Ethan's Quilava (DRI): Bonded by the Journey — once during your turn, you may search your deck for an Ethan's Adventure
//! card, reveal it, put it into your hand. Then, shuffle your deck. Combustion — 40.
//!
//! The search may find nothing (APR E-19) and then only the shuffle happens; the Ability is used either way, and its
//! once-per-turn usage record is made when it is used (id127: a failed search uses the Ability up for the turn). The card
//! goes into the hand as a PutIntoHand of the Ability's effect, revealed.
use crate::spec::prelude::*;


pub static SPEC: CardSpec = CardSpec {
    class: "EthansQuilava",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::Name("Ethan's Adventure"), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: true,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
