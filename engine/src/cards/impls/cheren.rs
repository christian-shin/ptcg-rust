//! Cheren (EPO / ASC): draw 3 cards.
//!
//! Twinleaf throws CANNOT_PLAY_THIS_CARD when the deck is empty.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cheren", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        draw_cards(g, p, 3)?;
    }
    Ok(())
}
