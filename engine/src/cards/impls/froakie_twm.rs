//! Froakie (TWM): Flock - search your deck for up to 2 Froakie and put them onto your Bench. Then, shuffle your deck. Flop - 10.
//!
//! The attack's search runs with room on the Bench and while a Froakie can still be in the deck: with all 4 Froakie in zones
//! that are known to be out of it, the attack is usable and finds nothing (no search, no shuffle). The Pokémon found are put on
//! the Bench (each an EnterPlay), then one shuffle after the search (APR E-19, id2303).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Froakie@TWM",
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
