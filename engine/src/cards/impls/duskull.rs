//! Duskull (SFA): Come and Get You — put up to 3 Duskull from your discard
//! pile onto your Bench. Mumble — 30.
//!
//! Twinleaf: the prompt is `max = min(empty bench slots, 3)`.
//! Fixed (phase 4b, R7C): "up to 3" in an attack may choose zero (rulings 1721,
//! 1778), so the prompt is `min: 0`; an attack that can do nothing (no Duskull in
//! the discard pile, no empty Bench slot) is still used and opens no prompt (ruling
//! 1790). It used to throw CANNOT_USE_POWER there and ask for at least 1.
//!
//! Events batch 2: each Duskull goes onto the Bench through the EnterPlay
//! event (mode effect, source the discard pile), so "puts onto the Bench"
//! effects see it: Risky Ruins places its counters (id2233). It used to be a
//! raw move that no card saw.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Duskull@SFA|PRE",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::All(&[Pred::Pokemon, Pred::Name("Duskull")]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                msg: "",
                cancel: false,
            }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
