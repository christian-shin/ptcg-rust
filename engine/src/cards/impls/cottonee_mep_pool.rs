//! Cottonee (MEP 18): Collect — draw a card. Ported so Whimsicott ex
//! (SV11W) can be evolved in pool games.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CottoneeMEPPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    draw_cards(g, p, 1)
}
