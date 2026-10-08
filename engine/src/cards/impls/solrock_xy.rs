//! Solrock (XY): Cosmic Spin — 10+, 30 more if Lunatone is on your Bench.
//! Solar Beam — 60.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Solrock@XY",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(30), when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Top(Pred::Name("Lunatone"))) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
