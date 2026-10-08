//! Lampent (PBL / M5): Spreading Light - search your deck for up to 3 Lampent
//! and put them onto your Bench, then shuffle.
//!
//! Twinleaf: max = min(3, empty Bench slots); nothing happens (no shuffle)
//! when the deck is empty or the Bench is full. Each chosen card is placed with
//! a PlayPokemonFromDeckEffect into the i-th empty slot; SHUFFLE_DECK follows.
//! Phase 4b R7E (ruling 336): with all 4 Lampent in known zones (discard pile,
//! in play) the attack is usable and fails without searching.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Lampent",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me), Cond::Not(&Cond::KnownCopies { who: Who::Me, name: "Lampent", at_least: 4 })]), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Name("Lampent")]), bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::OpenBench(Who::Me), &Num::Lit(3)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                msg: "",
                cancel: false,
                shuffle_first: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
