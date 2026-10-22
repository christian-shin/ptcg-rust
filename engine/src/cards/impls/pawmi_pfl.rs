//! Pawmi (PFL 32): Growl - during your opponent's next turn, attacks used by the Defending Pokémon do 30 less damage (before applying Weakness and Resistance). Tiny Charge - 10.
//!
//! Arms the lasting `DealsLessDamage(30)` on the Defending Pokémon (Chikorita ASC 8's Growl).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PawmiPFL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::DealsLessDamage(30) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
