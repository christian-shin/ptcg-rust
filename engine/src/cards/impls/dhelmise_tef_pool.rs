//! Dhelmise (TEF 19): Spinning Attack — 30. Steel Anchor — 80+; 80 more damage if you have any [M] Pokémon on your Bench.
//!
//! Steel Anchor adds 80 before the damage when any Benched Pokémon of yours has [M] among its types after effects
//! (`Cond::AnySlot` over the Bench with `SlotPred::TypeIs`, the checked type read, never the printed type).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DhelmiseTEF",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(80), when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::TypeIs(ct::METAL)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
