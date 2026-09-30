//! Rowlet (SFA): Add On — draw a card. Leafage — 10.
//!
//! Twinleaf: MOVE_CARDS (count 1, sourceCard) from deck to hand, with no
//! empty-deck check.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Rowlet@SFA", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1, me)
}
