//! Dunsparce (JTG): Trading Places — switch this Pokémon with 1 of your Benched Pokémon.
//!
//! An attack effect after the damage: `switch_self` (one ChangeActive by the attack). The class is pinned to the JTG card:
//! Dunsparce TEF is another card with the same name.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DunsparceJTG",
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
