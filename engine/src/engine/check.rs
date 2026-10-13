//! `check-effect.ts`: knock-outs, prizes, new Active, winner; plus the
//! check-state reducer for cost and energy checks.

use crate::effects::*;
use crate::game::{Game, R};
use crate::list::*;
use crate::state::*;
use crate::types::*;

// ---------------------------------------------------------------------------
// The cost and Energy computations' accumulators and their last steps (the derived layer's `attack_cost`,
// `retreat_cost`, `provided_energy`; the check effects' reducer runs the same last steps on the slow path)

/// An attack's cost or a Retreat Cost being computed (one computation for both, APR D-11 / D-12 / D-13; batch 8 C11b):
/// the printed cost, then what each declaration of the modifier kind does in the propagation order
/// (`passive::attack_cost_into`, `retreat_cost_into`), then [`finish_cost`].
#[derive(Clone, Copy, Debug)]
pub struct CostAcc {
    pub p: u8,
    /// The attack whose cost it is (`None`: the Active Pokémon's Retreat Cost).
    pub attack: Option<AttackRef>,
    /// The printed cost.
    pub cost: Cost,
    /// "Costs / is N more" (D-11): every one stacks.
    pub more: u8,
    /// "Costs / is N less" (D-12): every one stacks, calculated together with the "more" ones.
    pub less: u8,
    /// "Costs 1 Energy less" of any type (Sparkling Crystal): a "less" too, on any Energy of the cost.
    pub any_less: bool,
    /// "Has no Retreat Cost" (D-13).
    pub has_no: bool,
    /// "Ignore all [C] in the cost" (D-13).
    pub ignore_colorless: bool,
    /// "Ignore all Energy in the cost" / "can use this attack for ..." (D-13; R7F-11: a set cost is final).
    pub set_cost: Option<Cost>,
}

impl CostAcc {
    pub fn new(p: usize, attack: Option<AttackRef>, cost: Cost) -> CostAcc {
        CostAcc { p: p as u8, attack, cost, more: 0, less: 0, any_less: false, has_no: false, ignore_colorless: false, set_cost: None }
    }
}

/// The cost after every change (APR D-11, D-12, D-13): an effect that ignores all of it, sets it or says it has none
/// takes priority, and no "more" or "less" effect applies then (D-13); otherwise every "more" and "less" stacks and is
/// calculated together, one net sum of [C] floored at none, with no order between them (D-11 / D-12: a cost of 1 with
/// two "1 less" and one "1 more" is 0), then "1 Energy less" of any type (Sparkling Crystal: the Energy the attacker
/// provides covers each cost slot, one slot may stay open). For an attack, the lasting increase (Rillaboom's Drum
/// Beating) is one more "more".
pub fn finish_cost<F: crate::derived::Facts>(g: &mut F, mut acc: CostAcc) -> R<Cost> {
    let p = acc.p as usize;
    let a = g.g().st.players[p].active;
    if acc.has_no {
        return Ok(SVec::new());
    }
    if let Some(c) = acc.set_cost {
        return Ok(c);
    }
    if acc.ignore_colorless {
        let mut v: Cost = SVec::new();
        for t in acc.cost.iter() {
            if *t != ct::COLORLESS {
                v.push(*t);
            }
        }
        return Ok(v);
    }
    let lasting = match acc.attack {
        Some(_) => crate::engine::lasting::attack_cost_increase(&g.g().st, p, a),
        None => crate::engine::lasting::retreat_cost_increase(&g.g().st, p, a),
    };
    let net = acc.more as i32 + lasting.max(0) - acc.less as i32;
    if net > 0 {
        for _ in 0..net {
            acc.cost.push(ct::COLORLESS);
        }
    } else {
        for _ in 0..(-net) {
            match acc.cost.iter().position(|c| *c == ct::COLORLESS) {
                Some(i) => {
                    acc.cost.remove_at(i);
                }
                None => break,
            }
        }
    }
    let cost_now = acc.cost;
    if acc.any_less && cost_now.len() > 0 {
        let energy_map = g.provided_energy(p, SlotRef::new(p, a))?;
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
    Ok(acc.cost)
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

