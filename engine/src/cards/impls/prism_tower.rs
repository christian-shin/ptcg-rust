//! Prism Tower (CRI / M4, stadium): once during each player's turn, that
//! player may discard 2 cards from their hand in order to draw a card.
//!
//! Twinleaf: the draw is a MOVE_CARDS of 1 card from the deck (no-op on an
//! empty deck). The empty-selection branch (restoring `stadiumUsedTurn`) is
//! unreachable: the prompt requires exactly 2 cards.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PrismTower", mask: mask(&[k::USE_STADIUM]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::UseStadium { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    let used = g.st.players[p].stadium_used_turn;
    if used == g.st.turn {
        bail!("CANNOT_USE_STADIUM");
    }
    if g.st.players[p].hand.len() < 2 {
        bail!("CANNOT_USE_STADIUM");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = used;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(2, 2, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if cards.is_empty() {
        g.st.players[p].stadium_used_turn = f.a[1];
        return Ok(());
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1, me)?;
    Ok(())
}
