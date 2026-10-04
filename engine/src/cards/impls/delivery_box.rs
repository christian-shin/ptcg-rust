//! Boxed Order (TEF): search your deck for up to 2 Item cards, reveal them,
//! and put them into your hand. Then, shuffle your deck. Your turn ends.
//!
//! Twinleaf quirk kept: the search prompt (min 1, max 2) is queued without
//! waiting and the EndTurnEffect is reduced immediately, before the answer.
//! When the answer arrives the cards move deck→hand, the reveal is queued
//! (when cards were chosen) and then a wait-less shuffle.
//! Fixed (phase 4b): min is 0 when the deck holds no Item (it was
//! unanswerable), and the deck is shuffled after the reveal (the shuffle was
//! only reached when no card was chosen).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "DeliveryBox", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Item as u8), ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let has_item = g.st.players[p].deck.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_trainer() && d.trainer_type == TrainerType::Item as u8
    });
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(if has_item { 1 } else { 0 }, 2, false), Cont::Card { card: me, frame: f });
    g.run_fx(Effect::EndTurn { p: p as u8 })?;
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    if !cards.is_empty() {
        let id = g.player_id(1 - p);
        g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
