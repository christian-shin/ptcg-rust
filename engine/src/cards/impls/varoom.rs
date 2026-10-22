//! Varoom (SFA 43): Rigidify — during your opponent's next turn, this Pokémon takes 30 less damage from attacks (after applying
//! Weakness and Resistance).
//!
//! A lasting `TakesLessDamage(30, DamageSource::Any)` on this Pokémon until the end of the opponent's next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Varoom",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30, DamageSource::Any) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
