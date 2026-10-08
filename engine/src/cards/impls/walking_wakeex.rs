//! Walking Wake ex (TEF): Azure Wave — damage from attacks used by this
//! Pokémon isn't affected by any effects on your opponent's Active Pokémon.
//! Cathartic Roar — 120+; if your opponent's Active Pokémon is affected by a
//! Special Condition, this attack does 120 more damage.
//!
//! Twinleaf: on EVERY AttackEffect (any copy of this card, wherever it is)
//! whose player has a Pokémon named 'Walking Wake ex' Active: a probe
//! PowerEffect for this copy that throws skips the whole handler (including
//! Cathartic Roar's bonus); otherwise `effect.attack.shredAttack = true` is
//! written on the used attack's object (visible in the canonical `cards`
//! entry of the attacking card; nothing reads it any more) and
//! `ignoreDefenderEffects` is set (phase 4b R7B: the damage used to be added
//! straight to the Active, with Cathartic Roar's +120 then going through the
//! effects on the Defending Pokémon). Cathartic Roar adds 120 to the damage
//! when the opponent's Active has any Special Condition.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WalkingWakeex",
    // Azure Wave: damage from this Pokémon's attacks isn't affected by any effects on your opponent's Active Pokémon.
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::AttackFlags(AttackFlagsSpec { first_turn: false, shred: Some(SlotPred::Named("Walking Wake ex")) }) }],
    attacks: &[AttackSpec {
        index: 0,
        // Cathartic Roar: 120 more damage if your opponent's Active Pokémon is affected by a Special Condition (not while
        // the Ability is blocked: Twinleaf's handler returns early).
        steps: &[Step::before_damage(more_damage_if(120, Cond::All(&[Cond::Slot(OPP_ACTIVE, SlotPred::HasCondition), Cond::Not(&Cond::AbilityBlocked)])))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
