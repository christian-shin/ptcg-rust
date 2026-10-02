//! Great Tusk (TEF): Land Collapse — discard the top card of your opponent's
//! deck; if you played an Ancient Supporter from your hand this turn,
//! discard 3 more. Giant Tusk — 160.
//!
//! Twinleaf reads `player.ancientSupporter` (set by Explorer's Guidance /
//! Professor Sada's Vitality, cleared at the end of the turn).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GreatTusk", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, opp) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp),
            _ => return Ok(()),
        };
        move_count_from(g, ListRef::Deck(opp), ListRef::Discard(opp), 1, me)?;
        if g.st.players[p].ancient_supporter {
            move_count_from(g, ListRef::Deck(opp), ListRef::Discard(opp), 3, me)?;
        }
    }
    Ok(())
}
