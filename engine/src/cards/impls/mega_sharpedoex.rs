//! Mega Sharpedo ex (M2 / PFL 61): Greedy Fang — 70, draw 2 cards. Hungry
//! Jaws — 120+, 150 more if this Pokémon has any damage counters on it.
//!
//! Twinleaf fixed in phase 4b: Greedy Fang drew only 1 card (DRAW_CARDS 2
//! now: up to 2, MOVE_CARDS count without sourceCard), and the Hungry Jaws
//! branch tested attack index 0 after a block that always returned, so it
//! never applied; it is now on attack 1.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaSharpedoex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        draw_cards(g, p, 2)?;
    }
    if was_attack_used(g, e, 1, me) {
        let (p, a) = match *g.e(e) {
            Effect::Attack { p, .. } => (p as usize, g.st.players[p as usize].active),
            _ => return Ok(()),
        };
        if g.st.slot(p, a).damage > 0 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 150;
            }
        }
    }
    Ok(())
}
