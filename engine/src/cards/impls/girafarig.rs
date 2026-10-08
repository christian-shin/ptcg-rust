//! Girafarig (TEF): Psychic Assault — 20+, 10 more for each damage counter
//! on your opponent's Active Pokémon (Twinleaf adds `opponent.active.damage`).
//! Ported so Farigiraf ex (TEF) can evolve in check decks.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Girafarig@TEF",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageOn(OPP_ACTIVE), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
