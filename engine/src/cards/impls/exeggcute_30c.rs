//! Exeggcute (30C): Hypnosis — your opponent's Active Pokémon is now Asleep.
//!
//! Twinleaf has two `Exeggcute` classes; this port is bound to 30C.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Exeggcute@30C",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Asleep], Cause::Attack)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
