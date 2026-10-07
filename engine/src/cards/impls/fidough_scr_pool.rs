//! Fidough (SCR 66): Pleasant Aroma — search your deck for a Basic Pokémon and
//! put it onto your Bench, then shuffle. Stampede — 10.
//!
//! Twinleaf: returns without effect when the deck is empty or the Bench is
//! full (no throw); else SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH
//! ({ stage: BASIC }, { min: 0, max: 1 }).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "FidoughSCRPool", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() || empty_bench_slots(g, p).is_empty() {
            return Ok(());
        }
        let filter = Filter { stage: Some(Stage::Basic as u8), ..Filter::none() };
        search_deck_for_pokemon_to_bench(g, p, filter, ChooseCardsOpts::new(0, 1, true))?;
    }
    Ok(())
}
