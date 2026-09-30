//! Special Red Card (M4): usable only if the opponent has 3 or fewer Prize
//! cards left; they put their hand on the bottom of their deck and, if any
//! cards were put there, draw 3 cards.
//!
//! Twinleaf quirk kept: the hand is not shuffled (it goes to the bottom in
//! hand order).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SpecialRedCard", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn move_all(g: &mut Game, src: ListRef, dst: ListRef, me: CardId) -> R {
    g.run_fx(Effect::MoveCards { source: src, destination: dst, cards: None, count: None, to_top: false, to_bottom: false, skip_cleanup: false, source_card: me })?;
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let op = &g.st.players[o];
    let prizes = op.prizes[..op.prize_count as usize].iter().filter(|l| !l.is_empty()).count();
    if prizes > 3 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if !g.st.players[o].hand.is_empty() {
        let temp = g.alloc_temp(&[]);
        move_all(g, ListRef::Hand(o as u8), temp, me)?;
        move_all(g, temp, ListRef::Deck(o as u8), me)?;
        let n = 3.min(g.st.players[o].deck.len());
        draw_cards(g, o, n)?;
    }
    Ok(())
}
