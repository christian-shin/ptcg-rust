//! Seedot (MEG 13): Nap - heal 20 damage from this Pokémon. Seed Bomb - 20.
//!
//! A RemoveCounters (heal) on this Pokémon, caused by the attack (Cottonee WHT 4's Absorb).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SeedotMEG",
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
