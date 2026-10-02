//! Froakie (CRI / M4): Collect — draw a card. Water Gun — 10.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Froakie@Froakie M4", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            draw_cards(g, p as usize, 1)?;
        }
    }
    Ok(())
}
