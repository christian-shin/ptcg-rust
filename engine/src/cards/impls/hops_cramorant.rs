//! Hop's Cramorant (JTG): Fickle Spitting — 120; if your opponent doesn't
//! have exactly 3 or 4 Prize cards remaining, this attack does nothing.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HopsCramorant", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let left = g.st.players[opp].prize_left();
        if left != 3 && left != 4 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = 0;
            }
        }
    }
    Ok(())
}
