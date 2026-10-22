//! Reshiram (PFL 17): Combustion - 30. Burning Flare - 240; this Pokémon also does 60 damage to itself.
//!
//! One Damage event on the attacker after the main damage (Arrokuda DRI 62).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ReshiramPFL",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(60)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
