//! Jamming Tower (TWM / ASC, stadium): Pokémon Tools attached to each Pokémon
//! (both yours and your opponent's) have no effect.
//!
//! Twinleaf: throws CANNOT_USE_STADIUM on UseStadiumEffect. Every ToolEffect
//! finds the Pokémon holding the tool among the *effect player's own*
//! Pokémon (`effect.player.forEachPokemon(BOTTOM_PLAYER)`); when that target
//! exists and IS_STADIUM_EFFECT_BLOCKED (no stadium argument: NO_CARD here)
//! the Tool keeps working, otherwise the effect throws CANNOT_USE_POWER
//! (including when the tool sits on the other player's Pokémon).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "JammingTower",
    passives: &[
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::ToolEffects, ..PreventSpec::NONE }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
