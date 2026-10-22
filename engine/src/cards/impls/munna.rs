//! Munna (BLK 35): Rest - this Pokémon is now Asleep. Heal 30 damage from it. Mumble - 10.
//!
//! The Asleep condition is a GainCondition on this Pokémon, then the heal (Snorlax POR's Collapse, Skarmory DRI's Roost); the
//! heal happens even when this Pokémon can't be put to sleep.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Munna",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Asleep]), when: Cond::True })),
            Step::after_damage(heal_active(30)),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
