//! Delibird (MEG 105): Quick Gift (usable on the first turn when going first)
//! — search your deck for a card and put it into your hand, then shuffle.
//! Gentle Slap — 30.
//!
//! Twinleaf: SEARCH_DECK_FOR_CARDS_TO_HAND with an empty filter (so the pick is
//! not shown), min 1, max 1, no cancel; nothing happens with an empty deck.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "DelibirdMEGPool", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        search_deck_for_cards_to_hand(g, p, me, Filter::none(), ChooseCardsOpts::new(1, 1, false));
    }
    Ok(())
}
