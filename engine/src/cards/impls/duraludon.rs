//! Duraludon (SCR 106 / PRE 69): Hammer In — 30. Raging Hammer — 80+; 10 more damage for each damage counter on this Pokémon.
//!
//! Raging Hammer adds 10 per damage counter on the attacking (Active) Pokémon before the damage (`Num::DamageOn`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DuraludonSCR",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageOn(MY_ACTIVE), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
