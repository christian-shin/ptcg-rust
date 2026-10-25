//! Kingambit (ASC 148): Supreme Overlord - attacks used by this Pokémon do 30 more damage to your opponent's Active Pokémon for each
//! Prize card your opponent has taken (before applying Weakness and Resistance). Double-Edged Slash - 180; this Pokémon also does 50
//! damage to itself.
//!
//! A `DamageDealt` of 30 for the attacks of this Pokémon (`attacker: IsThisPokemon`), multiplied by the Prize cards the card owner's
//! opponent has taken (`times`). Double-Edged Slash: 50 damage to this Pokémon after the damage, Weakness and Resistance applying.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Kingambit",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(self_damage(50))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, times: Num::PrizesTaken(Who::Opp), attacker: SlotPred::IsThisPokemon, needs_damage: true, ..DamageDealtSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
