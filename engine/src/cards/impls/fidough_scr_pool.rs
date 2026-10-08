//! Fidough (SCR 66): Pleasant Aroma — search your deck for a Basic Pokémon and
//! put it onto your Bench, then shuffle. Stampede — 10.
//!
//! Twinleaf: returns without effect when the deck is empty or the Bench is
//! full (no throw); else SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH
//! ({ stage: BASIC }, { min: 0, max: 1 }).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "FidoughSCRPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me)]), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                msg: "",
                cancel: true,
                shuffle_first: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) }))], no: &[] })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
