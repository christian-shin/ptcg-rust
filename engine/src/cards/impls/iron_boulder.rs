//! Iron Boulder (SCR): Adjusted Horn — 170; if you don't have the same number
//! of cards in your hand as your opponent, this attack does nothing.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "IronBoulder", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, opp) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        if g.st.players[p].hand.len() != g.st.players[opp].hand.len() {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = 0;
            }
        }
    }
    Ok(())
}
