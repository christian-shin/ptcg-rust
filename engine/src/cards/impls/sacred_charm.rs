//! Sacred Charm (PFL; Twinleaf "Sacred Charm M2", tool): the Pokémon this card
//! is attached to takes 30 less damage from attacks from your opponent's
//! Pokémon that have any Abilities.
//!
//! Twinleaf bug kept: the handler tests `effect.source instanceof PokemonCard`,
//! but a DealDamageEffect's `source` is a PokemonCardList, so the reduction
//! never applies and the card has no effect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SacredCharm", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(_g: &mut Game, _me: CardId, _e: EffId) -> R {
    Ok(())
}
