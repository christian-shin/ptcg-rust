//! Lillie's Determination (M1L): shuffle your hand into your deck, draw 6
//! (8 with exactly 6 prizes left).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LilliesDetermination", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        let draw = if g.st.players[p].prize_left() == 6 { 8 } else { 6 };
        return shuffle_hand_into_deck_then_draw(g, p, me, draw);
    }
    Ok(())
}
