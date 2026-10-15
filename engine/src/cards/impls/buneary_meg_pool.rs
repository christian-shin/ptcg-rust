//! Buneary (MEG 107): Charm — during your opponent's next turn, attacks used by the Defending Pokémon do 20 less damage
//! (before Weakness and Resistance). Skip — 10.
//!
//! Charm arms the lasting `DealsLessDamage(20)` on the Defending Pokémon (shared with Chikorita ASC); it stays on that Pokémon
//! and ends with the opponent's next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BunearyMEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::DealsLessDamage(20) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
