//! Milcery (PFL 43): Draining Kiss - 10; heal 10 damage from this Pokémon.
//!
//! A RemoveCounters (heal) on this Pokémon after the damage (Cottonee WHT 4's Absorb).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MilceryPFL",
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
