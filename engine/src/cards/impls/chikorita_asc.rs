//! Chikorita (MC / ASC 8): Growl — during your opponent's next turn, attacks
//! used by the Defending Pokémon do 20 less damage (before W/R). Seed Bomb — 30.
//!
//! DEFENDING_POKEMON_DOES_LESS_DAMAGE: a ReduceDamageEffect setting the
//! opponent Active's `attackDamageReductionNextTurn`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Chikorita",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::DealsLessDamage(20) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
