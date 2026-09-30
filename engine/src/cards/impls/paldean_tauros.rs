//! Paldean Tauros (SSP): Spirited Tackle - 90+, 90 more damage if the
//! opponent's Active Pokémon is a Stage 1 Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PaldeanTauros", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        if let Some(c) = g.st.active_pokemon(o) {
            if g.st.cdef(c).stage == Stage::Stage1 as u8 {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 90;
                }
            }
        }
    }
    Ok(())
}
