//! Drapion (M3 / POR 52): Wrack Down — 60. Hazardous Tail — 100; this
//! Pokémon also does 70 damage to itself; the opponent's Active is now
//! Paralyzed and Poisoned.
//!
//! Rule: the conditions are effects of the attack: GainCondition(Poisoned) then
//! GainCondition(Paralyzed) on the opponent's Active, each with the attack's
//! cause, so Mist Energy and effect prevention stop them.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Drapion",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(70)),
                Step::after_damage(inflict(&[SpecialCondition::Poisoned])),
                Step::after_damage(inflict(&[SpecialCondition::Paralyzed])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
