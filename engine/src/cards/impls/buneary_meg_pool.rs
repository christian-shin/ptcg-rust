//! Buneary (MEG 107): Charm — during your opponent's next turn, attacks used
//! by the Defending Pokémon do 20 less damage (before W/R). Skip — 10.
//!
//! DEFENDING_POKEMON_DOES_LESS_DAMAGE (shared with Chikorita ASC).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BunearyMEGPool",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::DealsLessDamage(20) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
