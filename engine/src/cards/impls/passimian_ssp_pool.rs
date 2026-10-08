//! Passimian (SSP 111): Coordinated Throwing — 20 damage for each of your
//! Basic Pokémon in play (the top card of each slot).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PassimianSSPPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Basic), &Num::Lit(20)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
