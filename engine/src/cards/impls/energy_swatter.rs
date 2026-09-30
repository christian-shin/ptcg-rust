//! Energy Swatter (POR): your opponent reveals their hand, and you choose an
//! Energy card you find there and put it on the bottom of their deck.
//!
//! Twinleaf: a no-op hand→discard MOVE_CARDS of the card (it already sits in
//! the supporter pile), the reveal to the player is queued without waiting,
//! and with no Energy in the opponent's hand nothing else happens.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EnergySwatter", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    if g.st.players[o].hand.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    let id = g.player_id(p);
    g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    let any_energy = g.st.players[o].hand.iter().any(|c| g.st.cdef(c).is_energy());
    if !any_energy {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(o as u8), Filter::super_type(SuperType::Energy), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let o = 1 - f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if let Some(&c) = cards.first() {
        move_cards(g, ListRef::Hand(o as u8), ListRef::Deck(o as u8), &[c], me)?;
    }
    Ok(())
}
