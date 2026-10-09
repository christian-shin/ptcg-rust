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
        // This Pokémon can't be Asleep (any cause: events batch 4).
        modifier: Modifier::Prevent(PreventSpec::on(
            SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            EventPred::All(&[EventPred::Kind(EventKind::GainCondition), EventPred::Condition(SpecialCondition::Asleep)]),
        )),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
