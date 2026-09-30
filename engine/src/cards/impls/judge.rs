//! Judge (FST, SVI as POR): each player shuffles their hand into their deck and draws 4
//! cards (the player first; the opponent's sequence starts after the
//! player's draw, as Twinleaf's `afterDraw` callback).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Judge@FST|POR", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let pl = &g.st.players[p];
    if !pl.hand.iter().any(|c| c != me) && pl.deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    shuffle_hand_into_deck_then_draw_ex(g, p, me, me, 4, Some((me, f)))
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, _results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let o = 1 - f.a[0] as usize;
    shuffle_hand_into_deck_then_draw_ex(g, o, NO_CARD, me, 4, None)
}

