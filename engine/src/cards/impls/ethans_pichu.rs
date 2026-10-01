//! Ethan's Pichu (DRI): Sparking Draw — 30; draw a card (AFTER_ATTACK →
//! DRAW_CARDS(player, 1)).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EthansPichu", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, .. } = *g.e(e) {
            draw_cards(g, p as usize, 1)?;
        }
    }
    Ok(())
}
