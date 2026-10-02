//! Ralts (M1S / ASC): Collect - draw a card. Headbutt - 10.
//!
//! Twinleaf: MOVE_CARDS from the deck to the hand with `count: 1` and
//! `sourceCard` this.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Ralts@MEG|ASC", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1, me)?;
    }
    Ok(())
}
