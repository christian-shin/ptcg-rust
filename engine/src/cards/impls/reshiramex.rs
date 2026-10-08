//! Reshiram ex (SV11W): Slash — 50. Blaze Burst — 130+; 50 more damage for
//! each Prize card your opponent has taken (6 - opponent.getPrizeLeft(), an
//! opponent-side count as written in Twinleaf). Discard an Energy from this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Reshiramex",
    attacks: &[
        AttackSpec { index: 0, steps: &[] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::PrizesTaken(Who::Opp), &Num::Lit(50)), when: Cond::True })),
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::Choose { count: 1, ty: crate::types::ct::COLORLESS } })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
