//! Maractus (BLK 8): Lively Needles — 20+; if this Pokémon was healed during this turn, this attack does 100 more damage. Pierce — 50.
//!
//! "Healed" is the record of a Heal event that removed damage counters from this Pokémon this turn (`SlotPred::HealedThisTurn`): a
//! heal that removed nothing (no damage counters on it) isn't healing (id2198). The record ends with the turn, whoever's turn the
//! heal was in.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MaractusBLK",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(100, Cond::Slot(SlotExpr::This, SlotPred::HealedThisTurn)))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
