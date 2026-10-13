//! Seviper (PFL 62): Excited Power - if you have any [D] Mega Evolution Pokémon ex in play, attacks used by this Pokémon do 120
//! more damage to your opponent's Active Pokémon (before applying Weakness and Resistance). Pitch-Black Fangs - 120.
//!
//! A `DamageDealt` declaration at the Attack stage for attacks used by this Pokémon (`Holder`), added to damage above 0, while
//! any of your Pokémon in play is a Mega Evolution Pokémon ex whose current type is [D] (`SlotPred::TypeIs`).
use crate::spec::prelude::*;
use crate::types::{ct, tag};

pub static SPEC: CardSpec = CardSpec {
    class: "Seviper",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            stage: DamageStage::Attack,
            amount: 120,
            attacker: SlotPred::Holder,
            side: Side::Any,
            needs_damage: true,
            // A [D] Mega Evolution Pokémon ex in play.
            guard: Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::TypeIs(ct::DARK), SlotPred::Top(Pred::PokemonEx), SlotPred::Tag(tag::POKEMON_SV_MEGA)])),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
