//! Fezandipiti (TWM): Adrena-Pheromone — if this Pokémon has any [D] Energy
//! attached and is damaged by an attack, flip a coin; if heads, prevent that
//! damage. Energy Feather — 30 damage for each Energy attached to this
//! Pokémon.
//!
//! Twinleaf: Adrena-Pheromone runs on every PutDamageEffect whose target slot
//! holds this card (not necessarily on top): it needs this card on top and the
//! attack phase, then IS_ABILITY_BLOCKED and a CheckProvidedEnergyEffect are
//! both evaluated with the OWNER as `player` (phase 4b: it used to be the
//! attacker). [D] or a rainbow unit counts. The damage must be positive; a
//! CoinFlipEffect (owner, no callback) decides.
//! Energy Feather counts every provided unit on the slot holding this card.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Fezandipiti", mask: mask(&[k::PUT_DAMAGE, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn provided(g: &mut Game, p: usize, slot: SlotRef) -> R<crate::effects::EnergyMap> {
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: slot, energy_map: SVec::new() })?;
    Ok(match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    })
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutDamage { b, damage, .. } = *g.e(e) {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) {
            let player = t.p as usize;
            if g.st.slot_pokemon(t.p as usize, t.s) != Some(me) || g.st.phase != GamePhase::Attack {
                return Ok(());
            }
            if is_ability_blocked(g, player, me, None) {
                return Ok(());
            }
            let map = provided(g, player, t)?;
            let has_dark = map.iter().any(|en| en.provides.contains(&ct::ANY) || en.provides.contains(&ct::DARK));
            if !has_dark {
                return Ok(());
            }
            if damage <= 0 {
                return Ok(());
            }
            let (c, _) = g.run_fx(Effect::CoinFlip { p: player as u8, callback: None, result: None, skip_reflip_stadium: false, skip_reflip_tool: false })?;
            if let Effect::CoinFlip { result: Some(false), .. } = c {
                return Ok(());
            }
            g.set_prevent(e, true);
            return Ok(());
        }
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let (sp, ss) = match g.st.find_pokemon_slot(me) {
            Some(x) => x,
            None => bail!("TypeError: findCardList"),
        };
        let map = provided(g, p, SlotRef::new(sp, ss))?;
        let mut n = 0;
        for en in map.iter() {
            n += en.provides.len() as i32;
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 30 * n;
        }
    }
    Ok(())
}
