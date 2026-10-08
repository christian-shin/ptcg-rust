//! Regigigas (PRE): Jewel Breaker — 100+, 230 more if your opponent's
//! Active Pokémon is a Tera Pokémon.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Regigigas",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(230), when: Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::Tag(tag::POKEMON_TERA))) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
