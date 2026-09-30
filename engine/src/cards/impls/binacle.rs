//! Binacle (M3 / POR 42): Double Draw — draw 2 cards (on AfterAttackEffect).
//! Scratch — 30.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Binacle", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, .. } = *g.e(e) {
            draw_cards(g, p as usize, 2)?;
        }
    }
    Ok(())
}
