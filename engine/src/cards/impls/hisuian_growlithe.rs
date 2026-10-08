//! Hisuian Growlithe (TWM): Blazing Destruction — discard a Stadium in play.
//! Take Down — 40, this Pokémon also does 10 damage to itself.
//!
//! Twinleaf: with no Stadium in play the attack does nothing (phase 4b: it used
//! to throw CANNOT_USE_ATTACK, but an attack can be used with no effect);
//! the Stadium goes to its owner's discard (MOVE_CARDS of the whole list).
//! Take Down's recoil is a DealDamageEffect aimed at the attacker's Active.
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
