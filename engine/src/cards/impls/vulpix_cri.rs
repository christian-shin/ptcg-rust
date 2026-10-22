//! Vulpix (CRI 8): Singe - your opponent's Active Pokémon is now Burned.
//!
//! A GainCondition(Burned) with the attack's cause (Fuecoco SSP 29).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VulpixCRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Burned])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
