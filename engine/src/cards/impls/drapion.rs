//! Drapion (M3 / POR 52): Wrack Down — 60. Hazardous Tail — 100; this
//! Pokémon also does 70 damage to itself; the opponent's Active is now
//! Paralyzed and Poisoned.
//!
//! Twinleaf: the conditions go through two AddSpecialConditionsEffects
//! (YOUR_OPPPONENTS_ACTIVE_POKEMON_IS_NOW_POISIONED / _PARALYZED), Poison
//! first. Fixed in phase 4b (R4): they were AddSpecialConditionsPowerEffects
//! (ADD_POISON / ADD_PARALYZED_TO_PLAYER_ACTIVE), an Ability-style effect that
//! Mist Energy and effect prevention don't stop.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Drapion",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(70)),
                Step::after_damage(inflict(&[SpecialCondition::Poisoned], Cause::Attack)),
                Step::after_damage(inflict(&[SpecialCondition::Paralyzed], Cause::Attack)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
