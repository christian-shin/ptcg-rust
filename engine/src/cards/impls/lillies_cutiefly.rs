//! Lillie's Cutiefly (JTG 66): Hold Still - heal 10 damage from this Pokémon.
//!
//! A RemoveCounters (heal) on this Pokémon, caused by the attack (Cottonee WHT 4).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LilliesCutiefly",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(10)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
