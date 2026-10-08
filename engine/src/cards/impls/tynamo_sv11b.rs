//! Tynamo (SV11B): Hold Still — heal 10 damage from this Pokémon.
//!
//! Twinleaf: a HealTargetEffect(effect, 10) targeting the player's Active.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tynamo@BLK|ASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(10, HealVia::Attack)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
