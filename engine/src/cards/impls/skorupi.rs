//! Skorupi (M3 / POR 51): Poison Jab — 20; the opponent's Active is now
//! Poisoned (a GainCondition(Poisoned) with the attack's cause, so Mist Energy
//! and effect prevention stop it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Skorupi",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Poisoned])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
