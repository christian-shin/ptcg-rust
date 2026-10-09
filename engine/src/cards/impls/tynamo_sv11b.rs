//! Tynamo (SV11B): Hold Still — heal 10 damage from this Pokémon.
//!
//! Rule: Hold Still is a RemoveCounters (heal) of 10 on the player's Active, caused by the attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tynamo@BLK|ASC",
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
