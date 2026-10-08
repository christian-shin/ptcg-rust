//! Croconaw (TEF): Reverse Thrust — 30; switch this Pokémon with 1 of your
//! Benched Pokémon.
//!
//! Twinleaf: AFTER_ATTACK for Reverse Thrust, then SWITCH_ACTIVE_WITH_BENCHED
//! for the effect's player. Fixed (R1-6): it used to react to the
//! AfterDamageEffect of the attack, so it never switched when no damage was
//! dealt (prevented, or 0 after Resistance).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Croconaw",
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
