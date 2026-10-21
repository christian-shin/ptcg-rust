//! Espurr (POR 33): Nap - heal 20 damage from this Pokémon. Stampede - 10.
//!
//! A RemoveCounters (heal) on this Pokémon, caused by the attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EspurrPOR",
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
