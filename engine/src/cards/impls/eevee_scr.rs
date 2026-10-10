//! Eevee (SCR): Call for Family — search your deck for a Basic Pokémon and
//! put it onto your Bench, then shuffle. Gnaw — 20.
//!
//! Twinleaf has several `Eevee` classes; this port is bound to SCR. With no
//! empty Bench slot the attack does nothing (no prompt, no shuffle); the
//! prompt is cancellable.
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
                    msg: "",
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
