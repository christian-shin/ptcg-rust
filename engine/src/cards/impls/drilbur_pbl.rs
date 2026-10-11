//! Drilbur (PBL / M5): Call for Family - search your deck for up to 2 Basic Pokémon and put them onto your Bench. Then,
//! shuffle your deck. Dig Claws - 50.
//!
//! An attack that searches with a full Bench or an empty deck is still usable: it does its damage and the search finds nothing
//! (the guard skips the search and the shuffle). Otherwise one Search op onto the Bench (each Pokémon an EnterPlay), then one
//! shuffle after the search (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Drilbur@PBL",
    // Call for Family: search your deck for up to 2 Basic Pokémon and put them onto your Bench,
    // then shuffle (the attack just does its damage with an empty deck or a full Bench).
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me)]),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
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
