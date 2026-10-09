//! Eevee ex (PRE): Rainbow DNA — this Pokémon can evolve into any Pokémon ex
//! that evolves from Eevee if you play it from your hand onto this Pokémon.
//! Coruscating Quartz — 200. Tera: prevent attack damage to this Pokémon
//! while it is on the Bench.
//!
//! Events batch 2: a `Permit` on its own evolving (`This(Role::Base)`) with a
//! Pokémon ex that evolves from Eevee, played from the hand by the rule,
//! lifting the "evolves from" limit. Grand Tree and Salvatore (from the deck)
//! aren't covered, per the printed "if you play it from your hand"
//! (docs/rulings/RULES.md). Other cards evolving from Eevee can't evolve it.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Eeveeex",
    passives: &[
        // Rainbow DNA.
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::Permit(PermitSpec {
                for_: EventPred::All(&[
                    EventPred::Kind(EventKind::Evolve),
                    EventPred::Source(RulesZone::Hand),
                    EventPred::Path(EvolvePath::Rule),
                    EventPred::This(Role::Base),
                    EventPred::Card(Pred::All(&[Pred::Tag(tag::POKEMON_EX_LOWER), Pred::EvolvesFrom("Eevee")])),
                ]),
                lifts: &[Limit::EvolvesFrom],
                while_: &[],
            }),
        },
        // Tera.
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
