//! Sacred Charm (PFL; Twinleaf "Sacred Charm M2", tool): the Pokémon this card
//! is attached to takes 30 less damage from attacks from your opponent's
//! Pokémon that have an Ability (after applying Weakness and Resistance).
//!
//! Twinleaf (fixed in phase 4b): on a PutDamageEffect whose target holds this
//! tool, during the attack phase, unless the tool is blocked for the owner or
//! the source belongs to the same player: a CheckPokemonPowersEffect on the
//! source's Pokémon (the old handler tested `effect.source instanceof
//! PokemonCard`, but the source is a PokemonCardList, so it never applied);
//! if any power is an Ability, the damage is reduced by 30 (floored at 0).
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
