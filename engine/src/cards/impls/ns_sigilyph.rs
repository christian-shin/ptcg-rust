//! N's Sigilyph (JTG): Psychic Sphere — 20. Victory Symbol — if you use this
//! attack when you have exactly 1 Prize card remaining, you win this game.
//!
//! Twinleaf: `endGame` with the winner chosen by `state.activePlayer`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsSigilyph", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].prize_left() == 1 {
            let owner = g.st.active_player;
            crate::engine::phase::end_game(g, if owner == 0 { WINNER_P1 } else { WINNER_P2 });
        }
    }
    Ok(())
}
