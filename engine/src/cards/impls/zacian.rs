//! Zacian (M2 / PFL): Limit Break — 50; if your opponent has 3 or fewer Prize
//! cards remaining, 90 more damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Zacian@Zacian M2", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        if g.st.players[o].prize_left() <= 3 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 90;
            }
        }
    }
    Ok(())
}
