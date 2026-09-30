//! Call Bell / Helper Bell (SSP): only if you go second, on your first turn:
//! search your deck for a Supporter card, reveal it, and put it into your
//! hand. Then, shuffle your deck.
//!
//! Twinleaf: playable only on game turn 2; no reveal prompt; the card moves
//! supporter→discard before the wait-less shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HelperBell", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.turn != 2 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Supporter as u8), ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let pu = p as u8;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    for c in cards {
        move_cards(g, ListRef::Deck(pu), ListRef::Hand(pu), &[c], me)?;
    }
    move_cards(g, ListRef::Supporter(pu), ListRef::Discard(pu), &[me], me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: pu });
    Ok(())
}
