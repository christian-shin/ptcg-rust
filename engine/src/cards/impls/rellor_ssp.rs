//! Rellor (SSP): Collect — draw a card. Rollout — 10.
//!
//! Twinleaf: returns early with an empty deck, otherwise MOVE_CARDS
//! (count 1, sourceCard) from deck to hand.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Rellor@SSP", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        return Ok(());
    }
    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1, me)
}
