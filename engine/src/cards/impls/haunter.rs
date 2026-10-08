//! Haunter (SSH): Nightmare — 20; your opponent's Active Pokémon is now
//! Asleep. Spooky Shot — 40.
//!
//! Twinleaf applies the sleep on AFTER_ATTACK via
//! ADD_SLEEP_TO_PLAYER_ACTIVE (an AddSpecialConditionsPowerEffect, so it is
//! not an attack effect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Haunter@SSH",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Asleep], Cause::Ability)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
