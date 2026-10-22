//! Pawniard (ASC 146): Push Down - 10; switch out your opponent's Active Pokémon to the Bench. (Your opponent chooses the new Active Pokémon.)
//!
//! A ChangeActive (SwitchOut) done to the opponent's Active Pokémon (Bayleef MEG 9's Push Down; APR C-04).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PawniardASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Switch(SwitchSpec { change: ActiveChange::SwitchOut, among: SwitchAmong::Bench, required: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
