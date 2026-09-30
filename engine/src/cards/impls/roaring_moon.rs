//! Roaring Moon (TEF): Vengeance Fletching — 70+, 10 more for each Ancient
//! card in your discard pile. Speed Wing — 120.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RoaringMoon", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let n = g.st.players[p as usize].discard.iter().filter(|c| g.st.cdef(*c).has_tag(tag::ANCIENT)).count() as i32;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 10 * n;
            }
        }
    }
    Ok(())
}
