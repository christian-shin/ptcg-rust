//! Diancie (SCR): Diffuse Reflection — 40 damage for each Special Energy
//! attached to all of your opponent's Pokémon. Power Gem — 60.
//!
//! Twinleaf counts Special Energy cards in `cards` of every opponent slot
//! (bench, then Active) and sets `effect.damage = 40 * count`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Diancie", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[opp];
        let mut n = 0;
        for &s in pl.bench.iter().chain(std::iter::once(&pl.active)) {
            for c in pl.slots[s as usize].cards.iter() {
                let d = g.st.cdef(c);
                if d.is_energy() && d.energy_type == EnergyType::Special as u8 {
                    n += 1;
                }
            }
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 40 * n;
        }
    }
    Ok(())
}
