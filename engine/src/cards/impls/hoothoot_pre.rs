//! Hoothoot (PRE): Insomnia — this Pokémon can't be Asleep. Tackle — 20.
//!
//! Twinleaf has several `Hoothoot` classes; this port is bound to PRE.
//! Fixed in phase 4b (R4): on an AddSpecialConditionsEffect or
//! AddSpecialConditionsPowerEffect that includes Asleep and whose target's top
//! Pokémon is this card, Asleep is removed from the effect (the effect is
//! prevented when it was the only condition) unless the Ability is blocked
//! for the owner. It used to prevent every such effect, whatever its target,
//! from any zone and without an Ability-lock check.
use crate::spec::prelude::*;
use crate::types::SpecialCondition;

pub static SPEC: CardSpec = CardSpec {
    class: "Hoothoot@PRE",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::ConditionImmunity(ConditionImmunitySpec {
            conds: &[SpecialCondition::Asleep],
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            prevent: true,
            sweep: false,
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
