//! Walking Wake ex (TEF): Azure Seas — damage from attacks used by this
//! Pokémon isn't affected by any effects on your opponent's Active Pokémon.
//! Catharsis Roar — 120+; if your opponent's Active Pokémon is affected by a
//! Special Condition, this attack does 120 more damage.
//!
//! An Ability lock (Iron Thorns ex' Initialization, ...) removes only Azure
//! Seas' effect; Catharsis Roar's bonus is attack text and stays (B-PC-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WalkingWakeex",
    // Azure Wave: damage from this Pokémon's attacks isn't affected by any effects on your opponent's Active Pokémon.
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::AttackFlags(AttackFlagsSpec { first_turn: false, shred: Some(SlotPred::Named("Walking Wake ex")) }) }],
    attacks: &[AttackSpec {
        index: 0,
        // Catharsis Roar: 120 more damage if your opponent's Active Pokémon is affected by a Special Condition. It is
        // attack text: an Ability lock removes only Azure Seas' effect (the passive above), not this bonus.
        steps: &[Step::before_damage(more_damage_if(120, Cond::Slot(OPP_ACTIVE, SlotPred::HasCondition)))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
