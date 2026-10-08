//! Toxel (M2 / PFL 67): Call For Family — search your deck for up to 2 Basic
//! Pokémon and put them onto your Bench, then shuffle. Rascal Kick — 20.
//!
//! Twinleaf: nothing happens (no shuffle) with a full Bench; the empty
//! slots are fixed when the attack starts; the shuffle has no animation wait.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Toxel",
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
                        msg: "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
                        cancel: false,
                        shuffle_first: false,
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
