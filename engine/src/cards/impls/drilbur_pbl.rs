//! Drilbur (PBL / M5): Call for Family — search your deck for up to 2 Basic
//! Pokémon and put them onto your Bench, then shuffle. Dig Claws — 50.
//!
//! SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH({ stage: BASIC },
//! { min: 0, max: 2 }): throws on an empty deck or a full Bench, except during
//! an attack (phase 4b R7E, rulings 336/337/1790: the attack is usable, the
//! search just fails; the prefab returns without searching, in both engines).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Drilbur@PBL", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let filter = Filter { stage: Some(Stage::Basic as u8), ..Filter::none() };
        search_deck_for_pokemon_to_bench(g, p, filter, ChooseCardsOpts::new(0, 2, true))?;
    }
    Ok(())
}
