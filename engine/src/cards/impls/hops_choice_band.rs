//! Hop's Choice Band (JTG, tool): attacks used by the Hop's Pokémon this card
//! is attached to cost [C] less and do 30 more damage to your opponent's
//! Active Pokémon (before applying Weakness and Resistance).
//!
//! Twinleaf: on CheckAttackCostEffect (holder is the player's Active) the
//! tool probe runs first, then the first [C] is removed if the Active is a
//! Hop's Pokémon (the handler always returns, so a cost check never reaches
//! the damage branch). On DealDamageEffect from the holder's slot, after the
//! probe, damage to the opponent's Active gets +30 for a Hop's holder when the
//! damage is above 0 (phase 4b: the guard was missing).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "HopsChoiceBand",
    passives: &[
        Passive { origin: RuleSource::Tool, modifier: Modifier::AttackCost(AttackCostSpec { change: CostChange::Reduce(Num::Lit(1)), subject: SlotPred::All(&[SlotPred::Holder, SlotPred::Tag(tag::HOPS)]), ..AttackCostSpec::DEFAULT }) },
        Passive { origin: RuleSource::Tool, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, attacker: SlotPred::All(&[SlotPred::Holder, SlotPred::Tag(tag::HOPS)]), needs_damage: true, ..DamageDealtSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
