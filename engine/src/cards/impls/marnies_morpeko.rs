//! Marnie's Morpeko (DRI): Spiky Wheel — 20+, 40 more for each [D] Energy
//! attached to this Pokémon (Twinleaf counts [D] and ANY in the provided
//! energy of `player.active`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MarniesMorpeko", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
    let mut n = 0;
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for em in energy_map.iter() {
            n += em.provides.iter().filter(|t| **t == ct::DARK || **t == ct::ANY).count() as i32;
        }
    }
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage += 40 * n;
    }
    Ok(())
}
