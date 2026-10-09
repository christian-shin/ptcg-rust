//! Applin (DRI 16): Mini Drain — 10; heal 10 damage from this Pokémon.
//!
//! Mini Drain is a RemoveCounters (heal) on this Pokémon, caused by the attack. Two `Applin` classes
//! exist; this port is bound to DRI.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Applin@DRI",
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
