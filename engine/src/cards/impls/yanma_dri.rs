//! Yanma (DRI): Whirlwind — switch out your opponent's Active Pokémon to the
//! Bench (your opponent chooses the new Active Pokémon). Razor Wing — 30.
//!
//! Rule: as Bayleef's Push Down: a ChangeActive (SwitchOut) done to the Active Pokémon (APR C-04, id2025).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Yanma@DRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Switch(SwitchSpec { change: ActiveChange::SwitchOut, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
