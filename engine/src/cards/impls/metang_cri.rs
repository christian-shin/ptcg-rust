//! Metang (CRI 60): Metal Claw - 30. Guard Press - 70; during your opponent's next turn, this Pokémon takes 30 less damage from attacks (after applying Weakness and Resistance).
//!
//! The lasting reduction `Lasting::TakesLessDamage` (Carkol ASC 119).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MetangCRI",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30, DamageSource::Any) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
