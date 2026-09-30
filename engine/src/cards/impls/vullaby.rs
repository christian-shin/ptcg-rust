//! Vullaby (RCL): Pluck — 10; before doing damage, discard all Pokémon Tools
//! from your opponent's Active Pokémon (one MOVE_CARDS per Tool).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Vullaby", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[o].active;
        let tools: Vec<CardId> = g.st.slot(o, a).tools.iter().collect();
        for t in tools {
            let a = g.st.players[o].active;
            move_cards(g, ListRef::Slot(o as u8, a), ListRef::Discard(o as u8), &[t], me)?;
        }
    }
    Ok(())
}
