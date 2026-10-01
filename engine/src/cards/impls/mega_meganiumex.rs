//! Mega Meganium ex (MC / ASC 10): Giant Bouquet — 70+; 50 more for each
//! [G] Energy attached to this Pokémon.
//!
//! Twinleaf: `CheckProvidedEnergyEffect(player)` (source defaults to
//! `player.active`); counts GRASS or ANY `provides` entries and sets
//! `effect.damage = 70 + 50 * count`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaMeganiumex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

/// Number of GRASS / ANY `provides` entries on `p`'s slot `s`.
pub fn grass_energy_count(g: &mut Game, p: usize, s: SlotId) -> R<i32> {
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, s), energy_map: SVec::new() })?;
    let mut count = 0;
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for em in energy_map.iter() {
            count += em.provides.iter().filter(|t| **t == ct::GRASS || **t == ct::ANY).count() as i32;
        }
    }
    Ok(count)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    let n = grass_energy_count(g, p, a)?;
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage = 70 + 50 * n;
    }
    Ok(())
}
