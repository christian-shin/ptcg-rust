//! Amoongus (SV11B / BLK 11): Dangerous Reaction — 30+; 120 more damage if
//! the opponent's Active Pokémon is affected by a Special Condition.
//! Seed Bomb — 60.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Amoongus",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(120), when: Cond::Slot(OPP_ACTIVE, SlotPred::HasCondition) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
