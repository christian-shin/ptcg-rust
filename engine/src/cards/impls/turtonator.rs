//! Turtonator (SSP): Fully Singe — discard an Energy from your opponent's
//! Active Pokémon ex. Steaming Stomp — 100.
//!
//! Twinleaf bug replicated: Fully Singe tests `activeCard.cardTag.includes(
//! CardTag.POKEMON_ex)` — the deprecated `cardTag` array, which is empty for
//! every pool Pokémon (only Excadrill ex SV11B sets it, and that card is not
//! in the pool) — so the attack never has an effect. Nothing to do here; the
//! port exists to document that.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Turtonator", mask: mask(&[]), reduce, resume: None, coin: None, can_play: None };

fn reduce(_g: &mut Game, _me: CardId, _e: EffId) -> R {
    Ok(())
}
