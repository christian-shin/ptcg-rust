//! Ethan's Typhlosion (DRI): Buddy Blast — 40+; 60 more damage for each
//! Ethan's Adventure card in your discard pile. Steam Artillery — 160.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EthansTyphlosion", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[p].discard.iter().filter(|c| g.st.cdef(*c).name == "Ethan's Adventure").count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 60 * n;
        }
    }
    Ok(())
}
