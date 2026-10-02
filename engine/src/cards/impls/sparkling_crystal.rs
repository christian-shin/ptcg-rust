//! Sparkling Crystal (SCR, ACE SPEC Pokémon Tool): when the Tera Pokémon this
//! card is attached to uses an attack, that attack costs 1 Energy less.
//!
//! Twinleaf: on a CheckAttackCostEffect while the tool is on the player's
//! Active: a ToolEffect stub (not IS_TOOL_BLOCKED) must not throw, the Active
//! must be Tera; the provided Energy units (CheckProvidedEnergyEffect on the
//! Active) pay each printed cost slot in order ([C] with any unit, a typed
//! slot with a matching unit, else with a rainbow unit); if the covered slots
//! number at least cost length - 1 the cost becomes the covered units.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SparklingCrystal", mask: mask(&[k::CHECK_ATTACK_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, cost) = match *g.e(e) {
        Effect::CheckAttackCost { p, cost, .. } => (p as usize, cost),
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).tools.contains(me) {
        return Ok(());
    }
    let pokemon = g.st.slot_pokemon(p, a);
    if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
        return Ok(());
    }
    let c = match pokemon {
        Some(c) if g.st.cdef(c).has_tag(tag::POKEMON_TERA) => c,
        _ => return Ok(()),
    };
    let _ = c;
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
    let map = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    };
    let mut available: Vec<CardType> = Vec::new();
    for en in map.iter() {
        for t in en.provides.iter() {
            available.push(*t);
        }
    }
    if cost.len() > 0 {
        let mut contained: Vec<CardType> = Vec::new();
        for ct_ in cost.iter() {
            if *ct_ == ct::COLORLESS && !available.is_empty() {
                contained.push(available.remove(0));
                continue;
            }
            if let Some(i) = available.iter().position(|x| x == ct_) {
                contained.push(available.remove(i));
                continue;
            }
            if let Some(i) = available.iter().position(|x| *x == ct::ANY) {
                contained.push(available.remove(i));
            }
        }
        if contained.len() + 1 >= cost.len() {
            let mut out: crate::effects::Cost = SVec::new();
            for x in contained {
                out.push(x);
            }
            if let Effect::CheckAttackCost { cost, .. } = g.e_mut(e) {
                *cost = out;
            }
        }
    }
    Ok(())
}
