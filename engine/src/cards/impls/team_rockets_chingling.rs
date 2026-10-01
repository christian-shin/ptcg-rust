//! Team Rocket's Chingling (DRI): Ring Ring Noise — discard a random card
//! from your opponent's hand.
//!
//! Twinleaf: on AFTER_ATTACK, `Chance.index(hand.length)` picks the card and
//! MOVE_CARDS discards it.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsChingling", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::AfterAttack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[opp].hand.len();
        if n == 0 {
            return Ok(());
        }
        let i = g.rng.index(n);
        let c = g.st.players[opp].hand.as_slice()[i];
        move_cards(g, ListRef::Hand(opp as u8), ListRef::Discard(opp as u8), &[c], me)?;
    }
    Ok(())
}
