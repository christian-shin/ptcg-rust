//! Brute Bonnet (TWM): Poison Spray — the opponent's Active is now Poisoned.
//! Relentless Punches — 50+, 50 more for each damage counter on the
//! opponent's Active.
//!
//! Twinleaf: Poison Spray uses ADD_POISON_TO_PLAYER_ACTIVE, an
//! AddSpecialConditionsPowerEffect (not the attack effect), which also sets
//! the target's poison/burn/sleep/confusion values to the defaults.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BruteBonnet",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Poisoned])),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::DamageOn(OPP_ACTIVE), &Num::Lit(5)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
