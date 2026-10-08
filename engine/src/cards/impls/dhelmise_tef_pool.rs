//! Dhelmise (TEF 19): Spinning Attack — 30. Steel Anchor — 80+; 80 more
//! damage if you have any [M] Pokémon on your Bench.
//!
//! Twinleaf: each non-empty Bench slot with a Pokémon card gets a
//! CheckPokemonTypeEffect (`some` stops at the first [M]).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DhelmiseTEFPool",
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
