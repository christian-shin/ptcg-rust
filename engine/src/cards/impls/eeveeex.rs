//! Eevee ex (PRE): Rainbow DNA — you can play Pokémon ex that evolve from
//! Eevee onto this Pokémon to evolve it. Coruscating Quartz — 200.
//! Tera: prevent attack damage to this Pokémon while it is on the Bench.
//!
//! Twinleaf: on every CheckTableStateEffect this card sets its own
//! `evolvesFromBase` to `['Eevee']` when it is in play and a stub-Ability
//! probe for its owner passes, else `[]`, so any card whose `evolvesFrom` is
//! 'Eevee' could evolve it. Fixed (R1-4): a PlayPokemonEffect whose target
//! holds this card as top Pokémon throws INVALID_TARGET when the played card
//! evolves from 'Eevee' and is not a Pokémon ex (Sylveon PRE 40 no longer can).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Eeveeex",
    passives: &[
        // Rainbow DNA: Pokémon ex that evolve from Eevee can evolve this Pokémon.
        Passive { origin: RuleSource::Ability, modifier: Modifier::EvolveFrom(EvolveFromSpec { names: &["Eevee"], only: Pred::Tag(tag::POKEMON_EX_LOWER) }) },
        // Tera.
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
