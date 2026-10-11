//! Eevee (SFA): Colorful Catch — search your deck for up to 3 Basic Energy cards of different types, reveal them, put them
//! into your hand. Then, shuffle your deck. Headbutt — 20.
//!
//! The most that can be taken is the number of distinct Energy types in the deck (at most 3). An empty deck can't be searched
//! (id840): nothing happens. The Search is a PutIntoHand of the attack's effect, revealed; the shuffle follows (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Eevee@SFA",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::If(IfSpec { cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::DistinctTypes(ZoneRef(Who::Me, Zone::Deck), Pred::BasicEnergy), &Num::Lit(3)) }, distinct_types: true, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
