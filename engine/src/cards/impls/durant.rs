//! Durant (WHT 70): Bite Together - 20+; if Durant is on your Bench, this attack does 20 more damage. Vise Grip - 50.
//!
//! The attack adds 20 before the damage when this Pokémon is on the Bench (it is there only when another effect lets a Benched
//! Pokémon use its attacks).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Durant",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(20, Cond::Slot(SlotExpr::This, SlotPred::IsBench)))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
