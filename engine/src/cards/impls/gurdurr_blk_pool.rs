//! Gurdurr (BLK 48): Low Kick — 30. Hammer Arm — 60; discard the top card of
//! your opponent's deck (DISCARD_TOP_X_OF_OPPONENTS_DECK).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GurdurrBLKPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 1, me) {
        return Ok(());
    }
    let o = match *g.e(e) {
        Effect::Attack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    move_count_from(g, ListRef::Deck(o as u8), ListRef::Discard(o as u8), 1, me)
}
