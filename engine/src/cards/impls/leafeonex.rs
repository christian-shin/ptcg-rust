//! Leafeon ex (PRE 6): Verdant Storm — 60 damage for each Energy attached to
//! all of your opponent's Pokémon. Moss Agate — 230; heal 100 damage from
//! each of your Benched Pokémon. Tera: no attack damage on the Bench.
//!
//! Twinleaf: Verdant Storm counts CheckProvidedEnergyEffect `energyMap`
//! entries (cards, not provided types) and sets `effect.damage`; Moss
//! Agate reduces a HealTargetEffect per Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Leafeonex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let mut energies = 0i32;
        for (s, _, _) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: o as u8, source: SlotRef::new(o, s), energy_map: SVec::new() })?;
            if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                energies += energy_map.len() as i32;
            }
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = energies * 60;
        }
    }
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let pu = p as usize;
        for (s, _, _) in for_each_pokemon(g, pu, PlayerType::TopPlayer).iter().copied() {
            if s == g.st.players[pu].active {
                continue;
            }
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(pu, s) };
            g.run_fx(Effect::HealTarget { b, damage: 100 })?;
        }
    }
    tera_rule(g, e, me);
    Ok(())
}
