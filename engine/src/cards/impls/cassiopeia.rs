//! Cassiopeia (SFA): only when it is the last card in your hand; search your
//! deck for up to 2 cards, put them into your hand, shuffle.
//!
//! Twinleaf: no Supporter-already-played check in the card (the core rejects
//! it), no move to the supporter pile, no preventDefault, no reveal; the
//! prompt requires 1-2 cards and the shuffle prompt has no wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cassiopeia", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let others = g.st.players[p].hand.iter().filter(|c| *c != me).count();
    if others != 0 || g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), ChooseCardsOpts::new(1, 2, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
