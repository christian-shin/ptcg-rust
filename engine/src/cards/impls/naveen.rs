//! Naveen (POR, supporter): discard any number of cards from your hand, then
//! draw cards until you have 5 cards in your hand.
//!
//! Twinleaf: fails on an empty deck only; the card moves to the supporter
//! pile (effect prevented) and the prompt lists the whole remaining hand
//! (no cancel). Fixed (phase 4b #45): the minimum is `max(0, hand - 4)`, so a
//! hand of 5 or more cards must discard enough to draw at least one card
//! (the card text: "if you can't draw any cards in this way, you can't use
//! this card"). DRAW_CARDS_UNTIL_CARDS_IN_HAND is a plain
//! `deck.moveTo(hand, n)` (no MoveCardsEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Naveen", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let others: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    let max = g.st.players[p].hand.len() as u8;
    let temp = g.alloc_temp(&others);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", temp, Filter::none(), ChooseCardsOpts::new(others.len().saturating_sub(4) as u8, max, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, NO_CARD)?;
    let _ = me;
    let n = 5usize.saturating_sub(g.st.players[p].hand.len());
    g.move_to(ListRef::Deck(p as u8), ListRef::Hand(p as u8), Some(n));
    Ok(())
}
