//! Toxicroak (MEG 79): Reckless Charge - 70; this Pokémon also does 20 damage to itself.
//!
//! Self damage after the damage (Arrokuda DRI 62; APR B-09).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ToxicroakMEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(self_damage(20))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
