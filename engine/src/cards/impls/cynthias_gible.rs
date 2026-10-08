//! Cynthia's Gible (DRI): Rock Hurl - 20; this attack's damage isn't
//! affected by Resistance.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasGible",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
