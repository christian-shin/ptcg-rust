//! Steven's Baltoy (DRI): Summoning Sign — search your deck for up to 2 Basic Steven's Pokémon and put them onto your Bench.
//! Then, shuffle your deck. Psychic Sphere — 20.
//!
//! With no empty Bench space the whole effect fails (id362: public knowledge), no search and no shuffle. Otherwise up to as
//! many Pokémon as there is room for (at most 2) enter play on the Bench from the deck (an EnterPlay caused by the attack), then
//! the deck is shuffled (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensBaltoy",
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
                            predicate: Pred::All(&[Pred::Basic, Pred::Tag(crate::types::tag::STEVENS)]),
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
