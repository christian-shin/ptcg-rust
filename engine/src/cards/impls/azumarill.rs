//! Azumarill (SSP): Glistening Bubbles - with a Tera Pokémon in play,
//! Double-Edge costs less. Double-Edge - 230; 50 damage to itself.
//!
//! Glistening Bubbles sets Double-Edge's cost to [P] while any of the owner's Pokémon in play is Tera. Double-Edge's
//! self-damage is one Damage event on this Pokémon (the attacker) with the attack as its cause, after the main damage;
//! no Weakness or Resistance applies and the opponent's effects on the attack's damage don't reduce it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Azumarill",
    // Glistening Bubbles: with any Tera Pokémon in play, you can use Double-Edge for [P].
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::SetCost(&[ct::PSYCHIC]),
            attack: Some(0),
            side: Side::Owner,
            guard: Cond::InPlay(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::POKEMON_TERA)),
            ..AttackCostSpec::DEFAULT
        }),
    }],
    // Double-Edge: 50 damage to itself.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(self_damage(50))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
