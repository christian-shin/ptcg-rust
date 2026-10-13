//! Ho-Oh (SSP): Flap - 50. Shining Blaze - 100+; 100 more if you have any Tera Pokémon on your Bench.
//!
//! Shining Blaze adds 100 to the damage when any Pokémon on your Bench has the Tera tag (`Cond::AnySlot` over the Bench); the
//! Active Pokémon, this one, doesn't count.
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
