//! Hisuian Growlithe (TWM): Blazing Destruction — discard a Stadium in play.
//! Take Down — 40, this Pokémon also does 10 damage to itself.
//!
//! Blazing Destruction does nothing when no Stadium is in play (an attack can be used with no effect); the Stadium goes
//! to its owner's discard pile. Take Down's recoil is one Damage event on the attacker with the attack as its cause,
//! after the main damage.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HisuianGrowlithe",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(DISCARD_STADIUM),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(self_damage(10)),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
