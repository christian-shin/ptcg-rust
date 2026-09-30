//! Rowlet (POR / M3): Find a Friend — search your deck for a Pokémon, reveal
//! it, put it into your hand, then shuffle. Tackle — 30.
//!
//! Twinleaf: SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_INTO_HAND with
//! { min: 0, max: 1 } (throws on an empty deck, so the attack is illegal).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Rowlet@POR", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    search_deck_for_pokemon_to_hand(g, p, Filter::none(), ChooseCardsOpts::new(0, 1, true))
}
