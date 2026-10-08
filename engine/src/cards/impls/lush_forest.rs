//! Forest of Vitality (MEG, stadium): each player's [G] Pokémon can evolve
//! into [G] Pokémon during the turn they play those Pokémon, except during
//! their first turn.
//!
//! Twinleaf: when a [G] Pokémon card is played (PlayPokemonEffect, reduced
//! before the core places it), every [G] Pokémon already in play for that
//! player (CheckPokemonTypeEffect) gets `pokemonPlayedTurn = turn - 1`. A
//! newly benched Basic is not in play yet, so it is only reset by a later
//! [G] play that turn.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LushForest",
    passives: &[
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec { what: BlockWhat::UseStadium }) },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::PlayedTurnReset(PlayedTurnResetSpec { card_type: ct::GRASS }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
