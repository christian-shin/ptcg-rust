//! Amoongus (SV11B / BLK 11): Dangerous Reaction — 30+; 120 more damage if
//! the opponent's Active Pokémon is affected by a Special Condition.
//! Seed Bomb — 60.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Amoongus", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[opp].active;
        if !g.st.slot(opp, a).special_conditions.is_empty() {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 120;
            }
        }
    }
    Ok(())
}
