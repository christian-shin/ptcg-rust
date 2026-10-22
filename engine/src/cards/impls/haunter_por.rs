//! Haunter (POR 49): Haunt — place 3 damage counters on your opponent's Active Pokémon.
//!
//! One PlaceCounters event on the Defending Pokémon caused by the attack: counters, not damage (APR C-07), so Weakness and
//! Resistance never apply.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HaunterPOR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(OPP_ACTIVE), counters: Num::Lit(3) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
