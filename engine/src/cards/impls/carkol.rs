//! Carkol (ASC 119): Guard Press - 20; during your opponent's next turn, this Pokémon takes 20 less damage from attacks (after applying Weakness and Resistance). Power Gem - 60.
//!
//! `Lasting::TakesLessDamage(20)` armed on this Pokémon (Clamperl DRI 54).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Carkol",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(20) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
