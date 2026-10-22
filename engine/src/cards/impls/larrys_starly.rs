//! Larry's Starly (ASC 168): Minor Errand-Running - search your deck for up to 2 Basic Energy cards, reveal them, and put them into your hand. Then, shuffle your deck. Glide - 10.
//!
//! One PutIntoHand of the revealed cards, then one shuffle (APR E-19; Axew BLK 68).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LarrysStarly",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::BasicEnergy,
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
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
