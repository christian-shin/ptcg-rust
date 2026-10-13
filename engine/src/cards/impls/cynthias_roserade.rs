//! Cynthia's Roserade (DRI): Glorious Cheer — attacks from your Cynthia's Pokémon deal 30 more damage to your opponent's
//! Active Pokémon.
//!
//! An Ability `DamageDealt` bonus at the Deal stage over the owner's Cynthia's Pokémon (this Pokémon included, and two
//! Roserade stack: JP FAQ Cynthia's Roserade), on damage dealt to the opponent's Active Pokémon only; an Ability lock turns it
//! off.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasRoserade",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, attacker: SlotPred::Tag(tag::CYNTHIAS), ..DamageDealtSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
