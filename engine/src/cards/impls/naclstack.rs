//! Naclstack (MEG 83): Rock Hurl - 50; this attack's damage isn't affected by Resistance.
//!
//! `AttackFlagKind::NoResistance` before the damage (Landorus POR).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Naclstack",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
