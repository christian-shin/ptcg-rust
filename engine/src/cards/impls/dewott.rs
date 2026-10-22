//! Dewott (WHT 22): Energized Shell - 30×; this attack does 30 damage for each Energy attached to this Pokémon.
//!
//! The damage is 30 times the Energy units this Pokémon provides (Rotom SSP 61).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dewott",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(30)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
