//! Steelix (M1L / MEG 93): Welcoming Tail — 40+; 200 more if you have exactly
//! 6 Prize cards remaining. Skull Bash — 140.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Steelix", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].prize_left() == 6 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 200;
            }
        }
    }
    Ok(())
}
