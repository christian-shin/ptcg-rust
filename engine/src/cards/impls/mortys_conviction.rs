//! Morty's Conviction (TEF, supporter): discard another card from your hand,
//! then draw a card for each of your opponent's Benched Pokémon.
//!
//! Twinleaf: the card moves to the supporter pile (effect prevented) before
//! the "another card" and empty-deck checks; the prompt lists the hand
//! without this card (1 required, no cancel); the cards are moved to the
//! hand with MOVE_CARDS `count` (no shuffle). Fixed in phase 4b (R4, Rulings
//! Compendium 851/1664): it also throws CANNOT_PLAY_THIS_CARD when the
//! opponent has no Benched Pokémon (it would discard a card for no effect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MortysConviction", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let others: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    if others.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let opl = &g.st.players[1 - p];
    if !opl.bench.iter().any(|b| !opl.slots[*b as usize].cards.is_empty()) {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let temp = g.alloc_temp(&others);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", temp, Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    let benched = g.st.players[o].bench.iter().filter(|b| !g.st.players[o].slots[**b as usize].cards.is_empty()).count();
    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), benched, me)
}
