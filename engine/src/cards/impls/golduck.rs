//! Golduck (ASC 40 / MEP 8): Damp (see Psyduck). Hydro Pump — 60+; 20 more
//! damage for each [W] provided by the Energy attached to this Pokémon
//! (CheckProvidedEnergy on the attacking player's Active).
use super::psyduck::reduce_damp;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Golduck@ASC|MEP", mask: mask(&[k::CHECK_POKEMON_POWERS, k::POWER, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    reduce_damp(g, me, e)?;
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
        let mut n = 0;
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for em in energy_map.iter() {
                n += em.provides.iter().filter(|t| **t == ct::WATER || **t == ct::ANY).count() as i32;
            }
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += n * 20;
        }
    }
    Ok(())
}
