//! Tapu Bulu (SFA): Wood Hammer — 220. This Pokémon also does 30 damage to itself.
//!
//! Twinleaf: a DealDamageEffect (Weakness/Resistance path) on the player's
//! Active, reduced from the AttackEffect handler (before the main damage).
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
