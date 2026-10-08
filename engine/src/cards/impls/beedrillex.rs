//! Beedrill ex (CRI / M4): Rumbling Bees — 110× the number of your Beedrill
//! and Beedrill ex in play.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Beedrillex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Top(Pred::OneOf(&[Pred::Name("Beedrill"), Pred::Name("Beedrill ex")]))), &Num::Lit(110)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
