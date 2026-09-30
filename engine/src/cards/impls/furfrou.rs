//! Furfrou (M3): Hand Trim — discard random cards from your opponent's hand
//! until they have 5 cards in their hand. Headbutt — 30.
//!
//! Twinleaf draws each discard with `Chance.index(hand.length)`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Furfrou", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        while g.st.players[opp].hand.len() > 5 {
            let n = g.st.players[opp].hand.len();
            let i = g.rng.index(n);
            let c = g.st.players[opp].hand.as_slice()[i];
            move_cards(g, ListRef::Hand(opp as u8), ListRef::Discard(opp as u8), &[c], me)?;
        }
    }
    Ok(())
}
