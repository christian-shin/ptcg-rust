//! Hand Trimmer (TEF): both players discard cards from their hand until they
//! each have 5 cards in hand (opponent first).
//!
//! Twinleaf: both ChooseCardsPrompts are queued at once (opponent's first),
//! each only when that hand has more than 5 cards; each callback discards
//! the chosen cards.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HandTrimmer", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    // Fixed (phase 4b, ruling 959): obviously no effect when neither player has more than 5 cards in hand.
    let mine = g.st.players[p].hand.iter().filter(|c| *c != me).count();
    if g.st.players[o].hand.len() <= 5 && mine <= 5 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    for q in [o, p] {
        let n = g.st.players[q].hand.iter().filter(|c| *c != me).count();
        if g.st.players[q].hand.len() > 5 {
            let k = (n as i32 - 5).max(0) as u8;
            let mut f = CardFrame::at(1);
            f.a[0] = q as i32;
            choose_cards(g, q, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(q as u8), Filter::none(), ChooseCardsOpts::new(k, k, false), Cont::Card { card: me, frame: f });
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let q = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Hand(q as u8), ListRef::Discard(q as u8), &cards, me)
}
