//! Duraludon (SCR): Hammer In — 30. Raging Hammer — 80+; 10 more damage for
//! each damage counter on this Pokémon.
//!
//! Twinleaf adds `player.active.damage` (the Active slot, not this card's own
//! slot) to the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Duraludon@SCR|PRE",
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
