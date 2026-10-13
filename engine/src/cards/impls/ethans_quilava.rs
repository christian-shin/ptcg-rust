//! Ethan's Quilava (DRI): Bonded by the Journey — once during your turn, you may search your deck for an Ethan's Adventure
//! card, reveal it, put it into your hand. Then, shuffle your deck. Combustion — 40.
//!
//! The search can be declined (a search for a kind) and then only the shuffle happens (APR E-19); the Ability counts as used,
//! and the once-a-turn usage record is made, when a card is taken. The card goes into the hand as a PutIntoHand of the Ability's
//! effect, revealed.
use crate::spec::prelude::*;


pub static SPEC: CardSpec = CardSpec {
    class: "EthansQuilava",
    // Adventure Bound: once during your turn, you may search your deck for an Ethan's Adventure,
    // reveal it and put it into your hand, then shuffle (it counts as used when you take the card).
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Copy)))],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::Name("Ethan's Adventure"), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "CHOOSE_CARD_TO_HAND",
                cancel: true,
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Chosen(0),
                yes: &[
                    Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy))),
                ],
                no: &[],
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
