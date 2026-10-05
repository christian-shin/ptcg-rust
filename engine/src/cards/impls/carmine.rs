//! Carmine (TWM): if you go first, you can use this card on your first turn.
//! Discard your hand and draw 5 cards.
//!
//! Twinleaf: throws when a Supporter was already played, or when both the
//! deck and the rest of the hand are empty (phase 4b: it used to throw on any
//! empty deck, although discarding the hand is an effect); the other hand
//! cards go to the discard pile in one MOVE_CARDS (no source card, only when
//! there are any), then DRAW_CARDS 5.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Carmine", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let cards: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    if g.st.players[p].deck.is_empty() && cards.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if !cards.is_empty() {
        move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, NO_CARD)?;
    }
    draw_cards(g, p, 5)
}
