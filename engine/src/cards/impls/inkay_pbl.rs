//! Inkay (PBL 51): Procurement - search your deck for an Item card, reveal it, and put it into your hand. Then, shuffle your
//! deck. Spinning Attack - 30.
//!
//! With a card in the deck, one search of 0 or 1 Item, revealed and put into the hand, then one shuffle (APR E-19; Celebi
//! MEG 12).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "InkayPBL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                    yes: &[
                        Step::new(Op::Search(SearchSpec {
                            pick: PickSpec {
                                predicate: Pred::Item,
                                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                                ..PickSpec::DEFAULT
                            },
                            destination: SearchDestination::Hand { reveal: true },
                            cancel: false,
                        })),
                        Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                    ],
                    no: &[],
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
