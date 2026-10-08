//! Bronzor (TEF): Mirror Attack — 10+, 30 more if your opponent's Active
//! Pokémon is a [P] Pokémon (via CheckPokemonTypeEffect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bronzor",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(30), when: Cond::Slot(OPP_ACTIVE, SlotPred::TypeIs(ct::PSYCHIC)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
