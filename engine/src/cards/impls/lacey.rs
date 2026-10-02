//! Lacey (SCR, supporter): shuffle your hand into your deck, then draw 4
//! cards (8 if your opponent has 3 or fewer Prize cards remaining).
//!
//! Twinleaf: SHUFFLE_HAND_INTO_DECK_THEN_DRAW excluding this card; the draw
//! count uses `getPrizeLeft() > 3`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Lacey", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let o = 1 - p;
    let draw = if g.st.players[o].prize_left() > 3 { 4 } else { 8 };
    shuffle_hand_into_deck_then_draw(g, p, me, draw)
}
