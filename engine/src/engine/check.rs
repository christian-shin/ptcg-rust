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
// The cost and Energy computations' accumulators and their last steps (the derived layer's `attack_cost`,
// `retreat_cost`, `provided_energy`; the check effects' reducer runs the same last steps on the slow path)

/// An attack's cost being computed: the printed cost, then each `AttackCost` declaration's change in the propagation
/// order (`passive::attack_cost_into`), then [`finish_attack_cost`].
#[derive(Clone, Copy, Debug)]
pub struct AttackCostAcc {
    pub p: u8,
    pub attack: AttackRef,
    pub cost: Cost,
    /// An effect set the cost (Kyurem's Plasma Bane, ...): final (R7F-11).
    pub set_cost: Option<Cost>,
    /// Every [C] ignored, those added later included (R7F-11).
    pub ignore_colorless: bool,
    /// "Costs [C] less": the reductions add up and apply once, with the increases (Advanced Rulebook D-11, D-12).
    pub reduction: u8,
    /// "Costs 1 Energy less" of any type (Sparkling Crystal), applied once after all the declarations.
    pub any_reduction: bool,
}

/// The last steps of an attack's cost: the lasting increase (Rillaboom's Drum Beating), the "[C] less" reductions, the
/// "1 Energy less" of any type against the Energy the attacker provides, and a set or ignored cost.
pub fn finish_attack_cost(g: &mut Game, mut acc: AttackCostAcc) -> R<Cost> {
    let p = acc.p;
    // attackCostIncreaseWhileActive / ignoreAttackCostCardTypes: not modeled.
    // attackCostIncreaseNextTurn: one more [C] per point (Rillaboom's Drum Beating).
    let a = g.st.players[p as usize].active;
    let n = g.st.slot(p as usize, a).attack_cost_increase_next_turn;
    {
        let AttackCostAcc { cost, reduction, .. } = &mut acc;
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
    let cost_now = acc.cost;
    if acc.any_reduction && cost_now.len() > 0 {
        let energy_map = crate::derived::provided_energy(g, p as usize, SlotRef::new(p as usize, a))?;
        let mut available: Vec<CardType> = Vec::new();
        for en in energy_map.iter() {
            for t in en.provides.iter() {
                available.push(*t);
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
            acc.cost = out;
        }
    }
    // A cost that an effect set or ignored is final (R7F-11; rulings 147,
    // 252, 1552, 1581, 1842): CheckAttackCostEffect.setCost / ignoreColorless.
    if let Some(c) = acc.set_cost {
        acc.cost = c;
    } else if acc.ignore_colorless {
        let mut v: Cost = SVec::new();
        for t in acc.cost.iter() {
            if *t != ct::COLORLESS {
                v.push(*t);
            }
        }
        acc.cost = v;
    }
    Ok(acc.cost)
}

/// A Retreat Cost being computed: the printed cost with the lasting increase (`retreat::check_retreat_cost_base`), then
/// each `RetreatCost` declaration's change in the propagation order (`passive::retreat_cost_into`), then
/// [`finish_retreat_cost`].
#[derive(Clone, Copy, Debug)]
pub struct RetreatCostAcc {
    pub p: u8,
    pub cost: Cost,
    /// "No Retreat Cost".
    pub no_cost: bool,
    /// "[C] less": the reductions add up and apply once, with the increases (D-11, D-12).
    pub reduction: u8,
}

/// The last steps of a Retreat Cost: "no Retreat Cost" takes priority over increases, whatever the handler order (phase
/// 4b, R2); otherwise the "[C] less" reductions (Air Balloon, Rescue Board) add up and are applied once, together with
/// the increases (Advanced Rulebook D-11, D-12). zeroRetreatCostNextTurn: not modeled.
pub fn finish_retreat_cost(acc: RetreatCostAcc) -> Cost {
    let mut cost = acc.cost;
    if acc.no_cost {
        cost.clear();
    } else {
        for _ in 0..acc.reduction {
            match cost.iter().position(|c| *c == ct::COLORLESS) {
                Some(i) => {
                    let mut out: Cost = SVec::new();
                    for (j, c) in cost.iter().enumerate() {
                        if j != i {
                            out.push(*c);
                        }
                    }
                    cost = out;
                }
                None => break,
            }
        }
    }
    cost
}

/// The last step of the Energy map of the Pokémon in `source`, after the `ProvidesEnergy` declarations: each plain
/// Energy card attached that isn't in it yet, in the spot's order, then the `energies` entries missing from it
/// (Pokémon-as-energy, or energy attached without leaving its list, e.g. Metang's Metal Maker): `(c as any).provides ||
/// []`, skipped if empty.
pub fn complete_energy_map(g: &Game, source: SlotRef, energy_map: &mut EnergyMap) {
    let slot = g.st.slot(source.p as usize, source.s);
    let mut add: SVec<EnergyEntry, 64> = SVec::new();
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
    for e in add.iter() {
        energy_map.push(*e);
    }
}

// ---------------------------------------------------------------------------
// checkStateReducer: the check effects' last steps (the slow path, `derived::slow_*`)

pub fn check_state_reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::CheckAttackCost { p, attack, cost, set_cost, ignore_colorless, reduction, any_reduction } => {
            let out = finish_attack_cost(g, AttackCostAcc { p, attack, cost, set_cost, ignore_colorless, reduction, any_reduction })?;
            if let Effect::CheckAttackCost { cost, .. } = g.e_mut(id) {
                *cost = out;
            }
            Ok(())
        }
        Effect::CheckRetreatCost { p, cost, no_cost, reduction } => {
            let out = finish_retreat_cost(RetreatCostAcc { p, cost, no_cost, reduction });
            if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(id) {
                *cost = out;
            }
            Ok(())
        }
        Effect::CheckProvidedEnergy { source, energy_map, .. } => {
            let mut map = energy_map;
            complete_energy_map(g, source, &mut map);
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(id) {
                *energy_map = map;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
