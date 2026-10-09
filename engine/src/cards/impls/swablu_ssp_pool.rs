//! Swablu (SSP 148): Disarming Voice — 10; your opponent's Active Pokémon is
//! now Confused (a GainCondition(Confused) with the attack's cause).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SwabluSSPPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Confused])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
