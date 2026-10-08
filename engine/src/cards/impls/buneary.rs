//! Buneary (PFL / M2): Run Around — switch this Pokémon with 1 of your
//! Benched Pokémon (AFTER_ATTACK). Kick — 20.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Buneary",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(switch_self()),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
