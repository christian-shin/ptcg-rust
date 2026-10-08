//! Slowpoke (MEP 86): Dopey Face — this Pokémon can't be Confused. Super Psy
//! Bolt — 50.
//!
//! Twinleaf: on an AddSpecialConditionsEffect or AddSpecialConditionsPowerEffect
//! whose conditions include Confused and whose target's top Pokémon is this
//! card, Confused is removed from the effect (the effect is prevented when it
//! was the only condition) unless the ability is blocked for the owner. Fixed
//! in phase 4b (R4): the whole effect used to be prevented, so a Burned and
//! Confused attack applied neither.
use crate::spec::prelude::*;
use crate::types::SpecialCondition;

pub static SPEC: CardSpec = CardSpec {
    class: "SlowpokeMEP86Pool",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::ConditionImmunity(ConditionImmunitySpec {
            conds: &[SpecialCondition::Confused],
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            prevent: true,
            sweep: false,
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
