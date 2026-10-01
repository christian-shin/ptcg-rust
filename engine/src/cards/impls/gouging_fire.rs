//! Gouging Fire (SSP): Knock Down — 30. Blazing Charge — 100+; 70 more if
//! your opponent has 4 or fewer Prize cards remaining.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GougingFire", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        if g.st.players[opp].prize_left() <= 4 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 70;
            }
        }
    }
    Ok(())
}
