//! Bayleef (MEG 9): Push Down — 50. Switch out your opponent's Active Pokémon
//! to the Bench (your opponent chooses the new Active Pokémon).
//!
//! Rule: the switch-out is a ChangeActive (SwitchOut) done to the opponent's Active Pokémon (APR C-04, id2025): when
//! an attack-effect protection on that Pokémon (Mist Energy, Unaware, ...) prevents it, the opponent isn't asked to
//! choose and nothing changes; a protected Benched Pokémon can still be brought in.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bayleef",
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
