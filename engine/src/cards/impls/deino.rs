//! Deino (SSP): Stomp Off — discard the top card of your opponent's deck.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Deino", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let o = match *g.e(e) {
        Effect::Attack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    move_count_from(g, ListRef::Deck(o as u8), ListRef::Discard(o as u8), 1, me)
}
