//! Girafarig (TEF 66): Psychic Assault — 20+; 10 more damage for each damage counter on your opponent's Active Pokémon.
//!
//! Psychic Assault adds the damage on the opponent's Active Pokémon (`Num::DamageOn`, in HP: 10 per counter) before the
//! damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GirafarigTEF",
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
