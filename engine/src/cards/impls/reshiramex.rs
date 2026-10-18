//! Reshiram ex (WHT 20): Slash: 50. Blazing Burst: 130+; this attack does 50 more damage for each Prize card your opponent
//! has taken. Discard an Energy from this Pokémon.
//!
//! The bonus counts the Prize cards the opponent has taken; the Energy is discarded after the damage (a LeavePlay event, this
//! attack its cause).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ReshiramEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::PrizesTaken(Who::Opp), &Num::Lit(50)), when: Cond::True })),
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: crate::types::ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
