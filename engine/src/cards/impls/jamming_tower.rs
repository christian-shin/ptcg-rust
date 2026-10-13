//! Jamming Tower (TWM / ASC, Stadium): Pokémon Tools attached to each Pokémon (both yours and your opponent's) have no effect.
//!
//! A `Prevent` of `PreventWhat::ToolEffects` while the Stadium works, read through `derived::tool_off` by every Tool declaration
//! (a Tool stays attached; its Ability-like effects, damage modifiers, HP and cost changes and triggers are off for both
//! players); scenarios jamming-tower-*, fix-*-jamming-tower-f1.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "JammingTower",
    passives: &[
        Passive { origin: RuleSource::Stadium, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::ToolEffects, ..PreventSpec::NONE }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
