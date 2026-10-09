//! Haunter (SSH): Nightmare — 20; your opponent's Active Pokémon is now
//! Asleep. Spooky Shot — 40.
//!
//! The Special Condition is an effect of the attack: Mist Energy and other
//! attack-effect protection prevent it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Haunter@SSH",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Asleep])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
