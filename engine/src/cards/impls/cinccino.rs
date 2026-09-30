//! Cinccino (TEF): Gentle Slap — 30. Special Roll — 70× the number of
//! Special Energy cards attached to this Pokémon.
//!
//! Twinleaf counts the Special Energy cards in the attacker's `player.active`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cinccino", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let n = g
            .st
            .slot(p, a)
            .cards
            .iter()
            .filter(|c| {
                let d = g.st.cdef(*c);
                d.is_energy() && d.energy_type == EnergyType::Special as u8
            })
            .count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = n * 70;
        }
    }
    Ok(())
}
