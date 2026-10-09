//! Togetic (SSP / ASC): Drain Kiss - 30; heal 30 damage from this Pokémon.
//!
//! Rule: Drain Kiss is a RemoveCounters (heal) of 30 on this Pokémon, caused by
//! the attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Togetic@SSP|ASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
