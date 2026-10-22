//! Seedot (TEF 3): Rigidify - during your opponent's next turn, this Pokémon takes 30 less damage from attacks (after applying Weakness and Resistance). Hang Down - 20.
//!
//! `Lasting::TakesLessDamage(30, DamageSource::Any)` armed on this Pokémon (an ApplyEffect event), like Clamperl DRI 54's Shell Press.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SeedotTEF",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30, DamageSource::Any) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
