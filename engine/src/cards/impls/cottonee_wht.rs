//! Cottonee (WHT 4): Absorb - 10; heal 10 damage from this Pokémon.
//!
//! A RemoveCounters (heal) on this Pokémon, caused by the attack, after the damage (Applin DRI 16's Mini Drain).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CottoneeWHT",
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
