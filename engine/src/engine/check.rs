//! `check-effect.ts`: knock-outs, prizes, new Active, winner; plus the
//! check-state reducer for cost and energy checks.

use crate::effects::*;
use crate::game::{Game, R};
use crate::list::*;
use crate::state::*;
use crate::types::*;

/// `CheckHpEffect`: constructing it resets `hpBonus`; returns the final HP.
pub fn check_hp(g: &mut Game, p: usize, s: SlotId) -> R<i32> {
    let card = g.st.slot_pokemon(p, s);
    if card.is_some() {
        g.st.players[p].slots[s as usize].hp_bonus = 0;
    }
    // With no card that reacts to CheckHp in the game, no session or trace to feed, the dispatch changes
    // nothing: the HP is the printed one (`PTCG_VERIFY_CACHE=1` dispatches anyway and compares).
    let quiet = !g.kinds_present.has(k::CHECK_HP) && g.copy_sessions.is_empty() && !g.trace_effects;
    let printed = hp_of(g, p, s, card);
    if quiet && !crate::game::verify_cache() {
        return Ok(printed);
    }
    g.run_fx_unit(Effect::CheckHp { p: p as u8, target: SlotRef::new(p, s), card })?;
    let hp = hp_of(g, p, s, card);
    if quiet {
        assert_eq!(hp, printed, "CheckHp changed the HP although no card reacts to it");
    }
    Ok(hp)
}

pub fn hp_of(g: &Game, p: usize, s: SlotId, card: Option<CardId>) -> i32 {
    match card {
        Some(c) => g.st.cdef(c).hp + g.st.slot(p, s).hp_bonus,
        None => 0,
    }
}

// ---------------------------------------------------------------------------
// checkStateReducer

pub fn check_state_reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::CheckAttackCost { p, .. } => {
            // attackCostIncreaseWhileActive / ignoreAttackCostCardTypes: not modeled.
            // attackCostIncreaseNextTurn: one more [C] per point (Rillaboom's Drum Beating).
            let a = g.st.players[p as usize].active;
            let n = g.st.slot(p as usize, a).attack_cost_increase_next_turn;
            if let Effect::CheckAttackCost { cost, reduction, .. } = g.e_mut(id) {
                for _ in 0..n.max(0) {
                    cost.push(ct::COLORLESS);
                }
                // "[C] less" effects (Counter Gain, Hop's Choice Band, Incineroar ex, Crabominable, Bloodmoon Ursaluna ex)
                // are applied once, together with the increases, whatever the handler order (Advanced Rulebook D-11, D-12).
                for _ in 0..*reduction {
                    match cost.iter().position(|c| *c == ct::COLORLESS) {
                        Some(i) => {
                            cost.remove_at(i);
                        }
                        None => break,
                    }
                }
            }
            // "Costs 1 Energy less" of any type (Sparkling Crystal): the Energy attached to the Pokemon covers each
            // cost slot, one slot may stay open.
            let (any_reduction, cost_now) = match *g.e(id) {
                Effect::CheckAttackCost { any_reduction, cost, .. } => (any_reduction, cost),
                _ => unreachable!(),
            };
            if any_reduction && cost_now.len() > 0 {
                let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: SlotRef::new(p as usize, a), energy_map: SVec::new() })?;
                let mut available: Vec<CardType> = Vec::new();
                if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                    for en in energy_map.iter() {
                        for t in en.provides.iter() {
                            available.push(*t);
                        }
                    }
                }
                let mut contained: Vec<CardType> = Vec::new();
                for ct_ in cost_now.iter() {
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
                if contained.len() + 1 >= cost_now.len() {
                    let mut out: Cost = SVec::new();
                    for x in contained {
                        out.push(x);
                    }
                    if let Effect::CheckAttackCost { cost, .. } = g.e_mut(id) {
                        *cost = out;
                    }
                }
            }
            if let Effect::CheckAttackCost { cost, set_cost, ignore_colorless, .. } = g.e_mut(id) {
                // A cost that an effect set or ignored is final (R7F-11; rulings 147,
                // 252, 1552, 1581, 1842): CheckAttackCostEffect.setCost / ignoreColorless.
                if let Some(c) = *set_cost {
                    *cost = c;
                } else if *ignore_colorless {
                    let mut v: Cost = SVec::new();
                    for t in cost.iter() {
                        if *t != ct::COLORLESS {
                            v.push(*t);
                        }
                    }
                    *cost = v;
                }
            }
            Ok(())
        }
        Effect::CheckRetreatCost { no_cost, reduction, .. } => {
            // zeroRetreatCostNextTurn: not modeled. A "no Retreat Cost" effect
            // (noRetreatCost) takes priority over increases, whatever the
            // handler order (phase 4b, R2). retreatCostIncreaseNextTurn
            // (Rillaboom's Drum Beating) is part of the base cost:
            // retreat::check_retreat_cost_base.
            if no_cost {
                if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(id) {
                    cost.clear();
                }
            } else if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(id) {
                // "Retreat Cost is [C] less" effects (Air Balloon, Rescue Board) add up and are applied once, together
                // with the increases, whatever the handler order (Advanced Rulebook D-11, D-12).
                for _ in 0..reduction {
                    match cost.iter().position(|c| *c == ct::COLORLESS) {
                        Some(i) => {
                            let mut out: Cost = SVec::new();
                            for (j, c) in cost.iter().enumerate() {
                                if j != i {
                                    out.push(*c);
                                }
                            }
                            *cost = out;
                        }
                        None => break,
                    }
                }
            }
            Ok(())
        }
        Effect::CheckProvidedEnergy { source, .. } => {
            let slot = *g.st.slot(source.p as usize, source.s);
            let mut add: SVec<EnergyEntry, 64> = SVec::new();
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e(id) {
                for c in slot.cards.iter() {
                    let d = g.st.cdef(c);
                    if d.is_energy() && !energy_map.iter().any(|e| e.card == c) && !add.iter().any(|e| e.card == c) {
                        let mut provides = SVec::new();
                        for &t in d.provides {
                            provides.push(t);
                        }
                        add.push(EnergyEntry { card: c, provides });
                    }
                }
                // `energies` entries missing from the map (Pokémon-as-energy, or
                // energy attached without leaving its list, e.g. Metang's
                // Metal Maker): `(c as any).provides || []`, skipped if empty.
                for c in slot.energies.iter() {
                    let d = g.st.cdef(c);
                    if !d.provides.is_empty() && !energy_map.iter().any(|e| e.card == c) && !add.iter().any(|e| e.card == c) {
                        let mut provides = SVec::new();
                        for &t in d.provides {
                            provides.push(t);
                        }
                        add.push(EnergyEntry { card: c, provides });
                    }
                }
            }
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(id) {
                for e in add.iter() {
                    energy_map.push(*e);
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
