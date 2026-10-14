//! Lampent (PBL / M5): Spreading Light - search your deck for up to 3 Lampent and put them onto your Bench. Then, shuffle your
//! deck.
//!
//! The search runs with room on the Bench and while a Lampent can still be in the deck (with all 4 in zones known to be out of
//! it, the attack is usable and finds nothing). The pick is at most the open Bench spots, up to 3; each Lampent put on the Bench is
//! an EnterPlay, then one shuffle after the search (APR E-19).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Lampent",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me), Cond::Not(&Cond::KnownCopies { who: Who::Me, name: "Lampent", at_least: 4 })]), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Name("Lampent")]), bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::OpenBench(Who::Me), &Num::Lit(3)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
