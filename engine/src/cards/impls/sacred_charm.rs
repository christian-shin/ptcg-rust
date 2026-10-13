//! Sacred Charm (PFL 93, Tool): the Pokémon this card is attached to takes 30 less damage from attacks from your opponent's
//! Pokémon that have an Ability (after applying Weakness and Resistance).
//!
//! A `DamageTaken` declaration on the holder at step 5 of the damage calculation (APR C-16): the attacker "has an Ability" is a
//! checked read (`SlotPred::HasAbility`): a Pokémon whose Ability is off (Watchtower, Flutter Mane) doesn't count (JP FAQ Sacred
//! Charm), and the damage is floored at 0.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SacredCharm",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 30, subject: SlotPred::Holder, source: SlotPred::HasAbility, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
