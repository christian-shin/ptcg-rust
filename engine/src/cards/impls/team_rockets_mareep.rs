//! Team Rocket's Mareep (DRI 72): Procurement — search your deck for an Item card, reveal it, and put it into your hand. Then, shuffle
//! your deck. Tiny Bolt — 10.
//!
//! The Item (none may be found) is revealed and put into the hand (a PutIntoHand event, cause: this attack), then the deck is
//! shuffled once (APR E-19); with an empty deck nothing is searched (as Team Rocket's Murkrow ASC 126).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsMareep",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Item, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: true },
                    cancel: true,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
