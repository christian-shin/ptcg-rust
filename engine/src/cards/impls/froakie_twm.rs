//! Froakie (TWM): Flock — search your deck for up to 2 Froakie and put them
//! onto your Bench, then shuffle. Flop — 10.
//!
//! Twinleaf: same shape as Chatot's A Capella (no empty-deck check; with a full
//! Bench the attack does nothing, with no search and no shuffle: phase 4b R7E,
//! ruling 337; also with all 4 Froakie in known zones, ruling 336; the prompt max is min(empty bench slots, 2)), with a name filter.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Froakie@Froakie TWM",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::BenchSpace(Who::Me), Cond::Cmp(Num::KnownCopies(Who::Me, "Froakie"), CmpOp::Lt, Num::Lit(4))]),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            chooser: Who::Me,
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::Basic, Pred::Name("Froakie")]),
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                            ..PickSpec::DEFAULT
                        },
                        destination: SearchDestination::Bench,
                        msg: "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
                        cancel: false,
                        shuffle_first: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
                ],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
