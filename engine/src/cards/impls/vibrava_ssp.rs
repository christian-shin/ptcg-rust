//! Vibrava (SSP 105): Screech - during your next turn, the Defending Pokémon takes 50 more damage from attacks (after
//! applying Weakness and Resistance). Cutting Wind - 50.
//!
//! `Lasting::TakesMoreDamage(50)` armed on the Defending Pokémon (Stunfisk ASC 62's Pouncing Trap): in force during your
//! next turn, after Weakness and Resistance, gone when the Pokémon moves to the Bench (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VibravaSSP",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesMoreDamage(50) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
