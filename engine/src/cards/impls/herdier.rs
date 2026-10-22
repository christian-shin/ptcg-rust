//! Herdier (WHT 75): Roar - switch out your opponent's Active Pokémon to the Bench. (Your opponent chooses the new Active Pokémon.) Lunge Out - 70.
//!
//! A SwitchOut done to the opponent's Active Pokémon (Bayleef MEG 9; APR C-04, id2025).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Herdier",
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
