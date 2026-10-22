//! Fennekin (CRI 11): Call for Family - search your deck for up to 2 Basic Pokémon and put them onto your Bench. Then, shuffle your deck. Steady Firebreathing - 10.
//!
//! With no room on the Bench the attack does nothing; otherwise the search puts up to 2 Basic Pokémon onto the Bench, then one shuffle (Emolga BLK 29).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Fennekin",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                cond: Cond::BenchSpace(Who::Me),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            chooser: Who::Me,
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::Basic,
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                            ..PickSpec::DEFAULT
                        },
                        destination: SearchDestination::Bench,
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
