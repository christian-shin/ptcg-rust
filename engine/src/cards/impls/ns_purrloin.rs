//! N's Purrloin (JTG): Pilfer - 30; your opponent reveals their hand, put a
//! card you find there on the bottom of their deck.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsPurrloin", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let o = 1 - p;
    if g.st.players[o].hand.is_empty() {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DECK", ListRef::Hand(o as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let o = 1 - f.a[0] as usize;
    let cards: Vec<CardId> = results.first().copied().unwrap_or(Res::Null).cards().to_vec();
    move_cards(g, ListRef::Hand(o as u8), ListRef::Deck(o as u8), &cards, me)
}
