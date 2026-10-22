//! Alcremie (PFL 44): Sweet Circle — 20×; this attack does 20 damage for each of your Pokémon in play.
//!
//! The damage is 20 times the Pokémon in the Active Spot and on the Bench, set before the damage (as Beheeyem TEF 74).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AlcremiePFL",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Any), &Num::Lit(20))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
