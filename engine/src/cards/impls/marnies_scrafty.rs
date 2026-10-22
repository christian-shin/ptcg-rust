//! Marnie's Scrafty (DRI 133): Rear Kick - 40. Wild Tackle - 160; this Pokémon also does 30 damage to itself.
//!
//! Self damage after the damage (Klang SCR 100).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarniesScrafty",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
