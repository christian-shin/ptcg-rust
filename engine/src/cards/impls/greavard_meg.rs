//! Greavard (MEG 65): Stampede - 10. Take Down - 40; this Pokémon also does 10 damage to itself.
//!
//! One Damage event on the attacker with the attack as its cause, after the main damage (Arrokuda DRI 62).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Greavard",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(10)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
