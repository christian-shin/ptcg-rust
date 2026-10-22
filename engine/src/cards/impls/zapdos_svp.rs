//! Zapdos (SVP 157): Follow-Up Bolt - 20+; this attack does 10 more damage for each damage counter on your opponent's Active
//! Pokémon. Drill Peck - 80.
//!
//! A before-damage `DamageOp::Add` of the opponent's Active Pokémon's damage in HP (10 per counter, 1 x the HP of damage;
//! Morpeko ex PBL 55 counts 4 x for 40).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ZapdosSVP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageOn(OPP_ACTIVE), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
