//! Frillish (WHT / SV11W): Oceanic Gloom - 20; during your opponent's next
//! turn, they can't play any Item cards from their hand.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Frillish", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return opponent_cannot_play_cards(g, e, crate::effects::play_lock::ITEM);
    }
    Ok(())
}
