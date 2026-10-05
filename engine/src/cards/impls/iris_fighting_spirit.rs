//! Iris's Fighting Spirit (JTG / ASC): you can use this card only if you
//! discard another card from your hand. Draw cards until you have 6 cards in
//! your hand.
//!
//! Fixed (phase 4b #39): `reduceEffect` throws SUPPORTER_ALREADY_PLAYED after
//! another Supporter and CANNOT_PLAY_THIS_CARD without another card in the
//! hand; the discard prompt is min 1, max 1, no cancel (it used to allow an
//! empty choice, which played the card for nothing). The Supporter has left
//! the hand by the time the prompt is answered, so only other cards are
//! listed. Then DRAW_CARDS_UNTIL_CARDS_IN_HAND (plain `deck.moveTo(hand, n)`)
//! unless the hand already has 6 or more cards.
//!
//! R7C: unplayable when it would draw nothing (empty deck, or 7 or more other cards in
//! the hand so that 6 are left after discarding one; rulings 851, 959, 1037).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "IrisFightingSpirit", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        if !g.st.players[p].hand.iter().any(|c| c != me) {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        // "Draw cards until you have 6 cards in your hand": a card that would draw nothing (empty
        // deck, or 6 or more cards left after discarding another one) can't be played (rulings 851, 959, 1037).
        if g.st.players[p].deck.is_empty() || g.st.players[p].hand.iter().filter(|c| *c != me).count() >= 7 {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
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
