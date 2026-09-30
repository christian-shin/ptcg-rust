//! Sandy Shocks (TEF): Magnetic Burst — 20+; 70 more if you have 3 or more
//! Energy in play; this attack's damage isn't affected by Weakness.
//! Power Gem — 60.
//!
//! Twinleaf sets `ignoreWeakness` first, then counts `provides` entries of
//! a CheckProvidedEnergyEffect per in-play Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SandyShocks", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if let Effect::Attack { ignore_weakness, .. } = g.e_mut(e) {
            *ignore_weakness = true;
        }
        let mut n = 0usize;
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, s), energy_map: SVec::new() })?;
            if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                n += energy_map.iter().map(|x| x.provides.len()).sum::<usize>();
            }
        }
        if n >= 3 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 70;
            }
        }
    }
    Ok(())
}
