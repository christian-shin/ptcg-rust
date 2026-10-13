//! Cinccino (TEF): Gentle Slap — 30. Special Roll — 70× the number of Special Energy cards attached to this Pokémon.
//!
//! Special Roll counts the Special Energy cards attached to this Pokémon (`EnergyUnit::SpecialEnergyCards`: the text says
//! "Special Energy cards", so cards, not Energy provided).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cinccino",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::SpecialEnergyCards), &Num::Lit(70)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
