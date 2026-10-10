//! Black Kyurem ex (SSP): Ice Age — 90; if the opponent's Active Pokémon is a
//! [N] Pokémon (printed type), it is now Paralyzed. Black Frost — 250; this
//! Pokémon also does 30 damage to itself.
//!
//! Ice Age's Paralysis is a Special Condition event with the attack as its cause. Black Frost's 30 is a Damage event on
//! this Pokémon (no Weakness or Resistance: it is not an opponent's Pokémon, APR B-08), after the attack's own damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BlackKyuremex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Conditions(ConditionsSpec { target: OPP_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Paralyzed]), gate: Gate::None, when: Cond::Slot(OPP_ACTIVE, SlotPred::PrintedTypeIs(ct::DRAGON)) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
