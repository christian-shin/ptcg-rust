//! Ting-Lu (TWM 110): Ground Crack — 30; if a Stadium is in play, 30 damage
//! to each of your opponent's Benched Pokémon, then discard that Stadium.
//! Hammer In — 110.
//!
//! Twinleaf quirks kept: Ground Crack only sets the card-object flag
//! `discardedStadiumCard`; the Stadium in play (whichever it is then) is
//! discarded on a later BetweenTurnsEffect, and the flag stays set until a
//! Stadium is found there.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TingLu", mask: mask(&[k::ATTACK, k::BETWEEN_TURNS]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if g.st.stadium_card().is_none() {
            return Ok(());
        }
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        g.st.cards[me as usize].discarded_stadium_card = true;
        for (s, _, _) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
            if s == g.st.players[o].active {
                continue;
            }
            put_damage(g, e, 30, SlotRef::new(o, s))?;
        }
    }
    if let Effect::BetweenTurns { .. } = *g.e(e) {
        if !g.st.cards[me as usize].discarded_stadium_card {
            return Ok(());
        }
        if let Some(stadium) = g.st.stadium_card() {
            if let Some(l) = g.st.locate(stadium) {
                let owner = l.owner().unwrap_or(0);
                g.run_fx(Effect::MoveCards {
                    source: l,
                    destination: ListRef::Discard(owner as u8),
                    cards: None,
                    count: None,
                    to_top: false,
                    to_bottom: false,
                    skip_cleanup: false,
                    source_card: me,
                })?;
            }
            g.st.cards[me as usize].discarded_stadium_card = false;
        }
    }
    Ok(())
}
