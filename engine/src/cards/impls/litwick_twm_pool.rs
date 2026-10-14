//! Litwick (TWM 36): Call for Family - search your deck for a Basic Pokémon and put it onto your Bench. Then, shuffle your deck.
//! Live Coal - 20.
//!
//! The attack just does its damage with an empty deck or a full Bench; otherwise one Search op onto the Bench (an EnterPlay), then
//! one shuffle after it (APR E-19). The search can be declined.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LitwickTWMPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me)]), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                cancel: true,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
