//! Carracosta (SCR 38): Primal Knowledge - attacks used by your Pokémon do 30 more damage to your opponent's Active Evolution
//! Pokémon (before applying Weakness and Resistance). Tidal Wave - 150.
//!
//! An Ability `DamageDealt` of 30 over the attacks of the owner's Pokémon, added in the attacker-side pass of the damage
//! calculation to the opponent's Active Pokémon when it is an Evolution Pokémon (`SlotPred::Evolution`), only when the attack does
//! damage (APR "does nothing", id675; Hop's Choice Band). It stacks with another Carracosta's.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CarracostaSCR",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, target: SlotPred::Evolution, needs_damage: true, ..DamageDealtSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
