//! Tapu Bulu (SFA): Wood Hammer — 220. This Pokémon also does 30 damage to itself.
//!
//! The self damage is a Damage event caused by the attack on this Pokémon (no Weakness or Resistance, APR B-08).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TapuBulu",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
