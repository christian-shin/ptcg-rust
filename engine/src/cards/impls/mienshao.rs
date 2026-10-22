//! Mienshao (WHT 53): Low Sweep - 40. Smash Uppercut - 80; this attack's damage isn't affected by Resistance.
//!
//! `AttackFlagKind::NoResistance` before the damage (Landorus POR).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Mienshao",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
