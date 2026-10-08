//! Dunsparce (JTG): Trading Places — switch this Pokémon with 1 of your
//! Benched Pokémon. Pinned to the JTG card: Dunsparce TEF is a different
//! Twinleaf class with the same name.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dunsparce@JTG",
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
