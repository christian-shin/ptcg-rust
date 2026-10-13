//! Moltres (M2 / PFL): Fighting Wings — 20+, 90 more if the opponent's Active
//! Pokémon is a Pokémon ex.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Moltres",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(90), when: Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::PokemonEx)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
