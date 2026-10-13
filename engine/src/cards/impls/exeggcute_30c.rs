//! Exeggcute (30C 1): Hypnosis — your opponent's Active Pokémon is now Asleep.
//!
//! An attack effect after the damage: `inflict(Asleep)`, a GainCondition caused by the attack (Mist Energy and the like
//! prevent it). The class is pinned to 30C: another Exeggcute card has other text.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Exeggcute@30C",
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
