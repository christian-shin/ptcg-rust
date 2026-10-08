//! Beheeyem (TEF 74): Cosmic Beatdown — 20 damage for each of your Pokémon in
//! play (one per occupied slot).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BeheeyemTEFPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Any), &Num::Lit(20)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
