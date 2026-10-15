//! Swinub (JTG): Call For Family - search your deck for up to 2 Basic Pokémon and put them onto your Bench. Then, shuffle your
//! deck. Lunge Out - 10.
//!
//! With a full Bench nothing happens (no search, no shuffle); otherwise one Search op onto the Bench, at most the open spots (each
//! Pokémon an EnterPlay), then one shuffle after it (APR E-19, id2303).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Swinub",
    attacks: &[AttackSpec {
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
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
