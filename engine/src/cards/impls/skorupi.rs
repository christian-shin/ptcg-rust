//! Skorupi (M3 / POR 51): Poison Jab — 20; the opponent's Active is now
//! Poisoned (YOUR_OPPPONENTS_ACTIVE_POKEMON_IS_NOW_POISIONED: an
//! AddSpecialConditionsEffect; fixed in phase 4b, R4: it was an Ability-style
//! AddSpecialConditionsPowerEffect via ADD_POISON_TO_PLAYER_ACTIVE).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Skorupi",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Poisoned])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
