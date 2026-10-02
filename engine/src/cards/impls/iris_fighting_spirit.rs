//! Iris's Fighting Spirit (JTG / ASC): you can use this card only if you
//! discard another card from your hand. Draw cards until you have 6 cards in
//! your hand.
//!
//! Twinleaf: the discard prompt over the hand is min 0, max 1, no cancel
//! (the card can be played with an empty choice, which does nothing; the
//! Supporter itself is still in the hand while choosing); then
//! DRAW_CARDS_UNTIL_CARDS_IN_HAND (plain `deck.moveTo(hand, n)`) unless the
//! hand already has 6 or more cards.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "IrisFightingSpirit", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if cards.is_empty() {
        return Ok(());
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    if g.st.players[p].hand.len() >= 6 {
        return Ok(());
    }
    let n = 6 - g.st.players[p].hand.len();
    g.move_to(ListRef::Deck(p as u8), ListRef::Hand(p as u8), Some(n));
    Ok(())
}
