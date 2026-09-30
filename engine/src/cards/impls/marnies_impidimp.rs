//! Marnie's Impidimp (DRI): Filch — draw a card. Corkscrew Punch — 10.
//!
//! Twinleaf draws in AFTER_ATTACK (after damage).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MarniesImpidimp", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, .. } = *g.e(e) {
            draw_cards(g, p as usize, 1)?;
        }
    }
    Ok(())
}
