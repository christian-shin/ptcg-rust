//! Eevee (SFA): Colorful Catch — search your deck for up to 3 Basic Energy
//! cards of different types, reveal them, and put them into your hand, then
//! shuffle. Headbutt — 20.
//!
//! Twinleaf has several `Eevee` classes; this port is bound to SFA. The
//! prompt's max is the number of distinct `provides[0]` among the deck's
//! Basic Energy (capped at 3). Fixed (R1-5): an empty deck no longer makes
//! the attack fail (nothing happens), the cards are revealed (ShowCards for
//! the opponent) and the deck is shuffled (the generator never resumed after
//! the prompt callback, so neither ever happened); the unreachable
//! CAN_ONLY_SELECT_TWO_DIFFERENT_ENERGY_TYPES throw is gone.
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
