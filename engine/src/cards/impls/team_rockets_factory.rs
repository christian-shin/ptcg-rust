//! Team Rocket's Factory (DRI, stadium): once during each player's turn, if
//! they played a "Team Rocket" Supporter from their hand this turn, they may
//! draw 2 cards.
//!
//! Twinleaf reads `player.rocketSupporter` (set by Team Rocket's Petrel);
//! the FACTORY_USED_MARKER is only bookkeeping (cleared at end of turn).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsFactory", mask: mask(&[k::USE_STADIUM, k::END_TURN]), reduce, resume: None, coin: None, can_play: None };

fn factory_used() -> crate::markers::MarkerName {
    crate::marker!("FACTORY_USED_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::UseStadium { p, .. } = *g.e(e) {
        if g.st.stadium_card() == Some(me) {
            let p = p as usize;
            if !g.st.players[p].rocket_supporter {
                bail!("CANNOT_USE_STADIUM");
            }
            // Fixed (phase 4b, rulings 1733/1734): the draw needs at least 1 card in the deck.
            if g.st.players[p].deck.is_empty() {
                bail!("CANNOT_USE_STADIUM");
            }
            draw_cards(g, p, 2)?;
            g.st.players[p].marker.add(factory_used(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        }
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(factory_used(), me) {
            m.remove_from(factory_used(), me);
        }
    }
    Ok(())
}
