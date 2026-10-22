//! Clamperl (DRI 54): Shell Press - 10; during your opponent's next turn, this Pokémon takes 10 less damage from attacks
//! (after applying Weakness and Resistance).
//!
//! `Lasting::TakesLessDamage(10, DamageSource::Any)` armed on this Pokémon (an ApplyEffect event), like Varoom SFA 43's Rigidify.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Clamperl",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(10, DamageSource::Any) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
