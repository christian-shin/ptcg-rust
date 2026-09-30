//! Galvantula (SFA): Compound Eyes — this Pokémon's attacks do 50 more
//! damage to your opponent's Active Pokémon that have an Ability. Shocking
//! Web — 50; 80 more if this Pokémon has any [L] Energy attached.
//!
//! Twinleaf quirks kept: Compound Eyes adds 50 on every DealDamageEffect of
//! this card's attack when the opponent's Active has any power (no target
//! check); Shocking Web sets the damage to 50 + 80 for *each* provided-energy
//! entry whose card is an Energy card printing [L] in `provides`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Galvantula@SFA", mask: mask(&[k::DEAL_DAMAGE, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::DealDamage { b, .. } = *g.e(e) {
        if b.attack != my_attack(g, me, 0) {
            return Ok(());
        }
        let p = b.player as usize;
        let o = 1 - p;
        let opp_active = g.st.slot_pokemon(o, g.st.players[o].active);
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        if let Some(c) = opp_active {
            if !g.st.cdef(c).powers.is_empty() {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 50;
                }
            }
        }
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let source = SlotRef::new(p, g.st.players[p].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source, energy_map: SVec::new() })?;
        let mut damage = 50;
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for em in energy_map.iter() {
                let d = g.st.cdef(em.card);
                if d.is_energy() && d.provides.contains(&ct::LIGHTNING) {
                    damage += 80;
                }
            }
        }
        if let Effect::Attack { damage: d, .. } = g.e_mut(e) {
            *d = damage;
        }
    }
    Ok(())
}
