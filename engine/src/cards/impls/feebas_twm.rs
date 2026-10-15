//! Feebas (TWM): Flail — 10×; 10 damage for each damage counter on this Pokémon.
//!
//! The damage is set before the damage step to the damage on the attacking (Active) Pokémon (`Num::DamageOn`, in HP: 10 per
//! counter).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FeebasTWM",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::DamageOn(MY_ACTIVE), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
