//! Mega Sharpedo ex (M2 / PFL 61): Greedy Fang — 70, draw 2 cards. Hungry
//! Jaws — 120+, 150 more if this Pokémon has any damage counters on it.
//!
//! Twinleaf quirks kept: Greedy Fang moves only 1 card (MOVE_CARDS count 1,
//! no sourceCard) and does nothing on an empty deck; the Hungry Jaws branch
//! also tests attack index 0 (unreachable), so Hungry Jaws is a plain 120.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaSharpedoex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        move_count(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1)?;
    }
    Ok(())
}
