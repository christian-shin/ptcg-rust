//! Wailord ex (PBL 16): Surf - 120. Falling Down - 270; this Pokémon is now Asleep.
//!
//! Falling Down puts this Pokémon to sleep after the damage (a Special Condition event on itself; Snorlax POR 63's
//! Collapse).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WailordEx",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Asleep]), when: Cond::True }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
