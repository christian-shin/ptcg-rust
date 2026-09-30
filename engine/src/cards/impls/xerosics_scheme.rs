//! Xerosic's Machinations (SFA): your opponent discards cards from their
//! hand until they have 3 cards in their hand.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "XerosicsScheme", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let n = g.st.players[o].hand.len();
    if n <= 3 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let amount = (n - 3) as u8;
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut f = CardFrame::at(1);
    f.a[0] = o as i32;
    choose_cards(g, o, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(o as u8), Filter::none(), ChooseCardsOpts::new(amount, amount, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let o = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Hand(o as u8), ListRef::Discard(o as u8), &cards, me)?;
    Ok(())
}
