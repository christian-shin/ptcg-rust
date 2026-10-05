//! `retreat-effect.ts`.

use crate::effects::*;
use crate::energy;
use crate::engine::game_effect::clear_effects;
use crate::engine::turn::switch_pokemon;
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug)]
pub struct RetreatCont {
    pub p: u8,
    pub bench_index: u8,
    pub move_to: ListRef,
}

fn assert_can_retreat(g: &Game, p: usize) -> R {
    // cannotRetreatWhileActive: not modeled.
    let a = g.st.slot(p, g.st.players[p].active);
    if a.cannot_retreat_next_turn {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    Ok(())
}

fn retreat_pokemon(g: &mut Game, p: usize, bench_index: u8) -> R {
    let pl = &g.st.players[p];
    let bench = match pl.bench.get(bench_index as usize) {
        Some(b) => *b,
        None => return Ok(()),
    };
    if g.st.active_pokemon(p).is_none() || g.st.slot_pokemon(p, bench).is_none() {
        return Ok(());
    }
    g.st.players[p].retreated_turn = g.st.turn;
    switch_pokemon(g, p, bench)
}

fn energy_cards(map: &EnergyMap) -> Vec<CardId> {
    let mut v = Vec::new();
    for e in map.iter() {
        for _ in 0..energy::unit_count(e.provides.as_slice()) {
            v.push(e.card);
        }
    }
    v
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    let (p, bench_index, ignore, move_to) = match *g.e(id) {
        Effect::Retreat { p, bench_index, ignore_status_conditions, move_retreat_cost_to } => {
            (p as usize, bench_index, ignore_status_conditions, move_retreat_cost_to)
        }
        _ => return Ok(()),
    };
    assert_can_retreat(g, p)?;
    let bench = match g.st.players[p].bench.get(bench_index as usize) {
        Some(b) => *b,
        None => crate::bail!("INVALID_TARGET"),
    };
    if g.st.slot(p, bench).cards.is_empty() {
        crate::bail!("INVALID_TARGET");
    }
    let active = g.st.players[p].active;
    let sp = g.st.slot(p, active).special_conditions;
    if (sp.contains(&(SpecialCondition::Paralyzed as u8)) || sp.contains(&(SpecialCondition::Asleep as u8))) && !ignore {
        crate::bail!("BLOCKED_BY_SPECIAL_CONDITION");
    }
    if g.st.players[p].retreated_turn == g.st.turn {
        crate::bail!("RETREAT_ALREADY_USED");
    }
    let mut cost: Cost = SVec::new();
    if let Some(c) = g.st.active_pokemon(p) {
        for &t in g.st.cdef(c).retreat {
            cost.push(t);
        }
    }
    let (e, _) = g.run_fx(Effect::CheckRetreatCost { p: p as u8, cost, no_cost: false })?;
    let cost = match e {
        Effect::CheckRetreatCost { cost, .. } => cost,
        _ => SVec::new(),
    };
    if cost.is_empty() {
        clear_effects(&mut g.st.players[p].slots[active as usize]);
        return retreat_pokemon(g, p, bench_index);
    }
    let (e, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, active), energy_map: SVec::new() })?;
    let map = match e {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    };
    if !energy::check_enough_energy(map.as_slice(), cost.as_slice()) {
        crate::bail!("NOT_ENOUGH_ENERGY");
    }
    if energy::check_exact_energy(map.as_slice(), cost.as_slice()) {
        let cards = energy_cards(&map);
        clear_effects(&mut g.st.players[p].slots[active as usize]);
        g.move_cards_to(ListRef::Slot(p as u8, active), &cards, move_to);
        return retreat_pokemon(g, p, bench_index);
    }
    if energy::all_provides_identical(map.as_slice()) {
        if let Some(sel) = energy::select_minimal_energy_for_cost(map.as_slice(), cost.as_slice()) {
            if !sel.is_empty() {
                let cards: Vec<CardId> = sel.iter().map(|e| e.card).collect();
                clear_effects(&mut g.st.players[p].slots[active as usize]);
                g.move_cards_to(ListRef::Slot(p as u8, active), &cards, move_to);
                return retreat_pokemon(g, p, bench_index);
            }
        }
    }
    let pid = g.player_id(p);
    g.prompt(
        pid,
        "CHOOSE_ENERGY_TO_PAY_RETREAT_COST",
        PromptKind::ChooseEnergy { energy: map, cost, allow_cancel: true },
        Cont::Retreat(RetreatCont { p: p as u8, bench_index, move_to }),
    );
    Ok(())
}

pub fn resume(g: &mut Game, rc: RetreatCont, res: Res) -> R {
    let p = rc.p as usize;
    let energy = match res {
        Res::Energy(e) => e,
        _ => return Ok(()),
    };
    let bench = match g.st.players[p].bench.get(rc.bench_index as usize) {
        Some(b) => *b,
        None => return Ok(()),
    };
    if g.st.active_pokemon(p).is_none() || g.st.slot_pokemon(p, bench).is_none() {
        return Ok(());
    }
    let cards: Vec<CardId> = energy.iter().copied().collect();
    let active = g.st.players[p].active;
    clear_effects(&mut g.st.players[p].slots[active as usize]);
    g.move_cards_to(ListRef::Slot(rc.p, active), &cards, rc.move_to);
    retreat_pokemon(g, p, rc.bench_index)
}
