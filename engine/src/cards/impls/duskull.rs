//! Duskull (SFA 40 / PRE): Come and Get You - put up to 3 Duskull from your discard pile onto your Bench. Mumble - 30.
//!
//! An attack's "up to 3" may be 0 (id2219, id2301); an attack that can do nothing (no Duskull in the discard pile, no
//! empty Bench spot) is still used and opens no prompt (id2313, id361). Each Duskull goes onto the Bench through the
//! EnterPlay event (mode effect, source the discard pile), so "puts onto the Bench" effects see it: Risky Ruins places its
//! counters (id2233).
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
