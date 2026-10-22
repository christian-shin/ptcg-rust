//! Finizen (PBL 21): Draining Fin - 20; heal 20 damage from this Pokémon.
//!
//! A RemoveCounters (heal) on this Pokémon, caused by the attack, after the damage (Cottonee WHT 4).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FinizenPBL",
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
