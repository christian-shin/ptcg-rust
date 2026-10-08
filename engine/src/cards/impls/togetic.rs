//! Togetic (SSP / ASC): Drain Kiss - 30; heal 30 damage from this Pokémon.
//!
//! Twinleaf: `new HealEffect(player, player.active, 30)` (a HealEffect, not
//! the attack-side HealTargetEffect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Togetic@SSP|ASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(30, HealVia::Effect)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
