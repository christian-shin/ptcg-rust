//! Eevee (SCR): Call for Family — search your deck for a Basic Pokémon and put it onto your Bench. Then, shuffle your deck.
//! Gnaw — 20.
//!
//! With no empty Bench space the whole effect fails, the search included (id362: public knowledge); the search itself can
//! be declined (a search for a kind). The Pokémon enters play from the deck (an EnterPlay caused by the attack), then the
//! shuffle follows (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Eevee@SCR",
    // Call for Family: search your deck for a Basic Pokémon and put it onto your Bench, then
    // shuffle (nothing happens with no empty Bench space; the search can be cancelled).
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::BenchSpace(Who::Me),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Bench,
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
