//! Seel (PFL 21): Bubble Drain - 20; heal 20 damage from this Pokémon.
//!
//! A RemoveCounters (heal) on this Pokémon after the damage (Cottonee WHT 4's Absorb).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SeelPFL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(20)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
