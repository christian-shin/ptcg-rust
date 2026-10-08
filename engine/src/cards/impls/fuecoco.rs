//! Fuecoco (SSP): Heat Burn — 20; the opponent's Active Pokémon is now Burned.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Fuecoco",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Burned], Cause::Attack)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
