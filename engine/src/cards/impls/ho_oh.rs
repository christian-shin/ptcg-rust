//! Ho-Oh (SSP): Flap — 50. Shining Blaze — 100+; 100 more if you have any Tera
//! Pokémon on your Bench.
//!
//! Fixed (phase 4b, W4): Twinleaf counted the Active Pokémon too.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "HoOh",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(100), when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Top(Pred::Tag(tag::POKEMON_TERA))) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
