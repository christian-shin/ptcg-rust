//! Rayquaza (ASC 153): Breakthrough Assault - 20+; if this Pokémon moved from your Bench to the Active Spot this turn, this attack does 90 more damage. Dragon Claw - 130.
//!
//! Read before the damage: `Cond::ThisMovedToActive` (Mega Lopunny ex PFL's Gale Thrust).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RayquazaASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(90, Cond::ThisMovedToActive)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
