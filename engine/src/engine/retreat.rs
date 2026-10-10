//! Retreating (APR A-03): the turn action's rules (once per turn, not Asleep or Paralyzed), the Retreat Cost, then
//! the ChangeActive event (`engine::change_active`, kind `Retreat`), which carries the checks ("can't retreat" locks,
//! the Pokémon's lasting "can't retreat") and the consequences (only the Pokémon moving to the Bench loses its
//! effects and Special Conditions: id356, id891).

use crate::effects::*;
use crate::energy;
use crate::engine::change_active::{self, ChangeActiveView};
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

/// The ChangeActive a retreat to the Bench slot `bench_index` makes (`None`: no Pokémon there).
pub fn retreat_change(g: &Game, p: usize, bench_index: u8) -> Option<ChangeActiveView> {
    let bench = *g.st.players[p].bench.get(bench_index as usize)?;
    g.st.slot_pokemon(p, bench)?;
    let cause = crate::cause::Cause::rule(crate::cause::RuleWhich::Retreat, p as u8);
    Some(ChangeActiveView::of(g, p, Some(bench), crate::spec::event::ActiveChange::Retreat, cause))
}

/// The Retreat Cost is paid: the Active Pokémon moves to the Bench (the ChangeActive event, checked before the cost).
fn retreat_pokemon(g: &mut Game, p: usize, bench_index: u8) -> R {
    let Some(c) = retreat_change(g, p, bench_index) else { return Ok(()) };
    if g.st.active_pokemon(p).is_none() {
        return Ok(());
    }
    g.st.players[p].retreated_turn = g.st.turn;
    change_active::produce(g, c)
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

/// Check only: the turn action's rules for retreating to the Bench slot `bench_index` (a Pokémon there, Special
/// Conditions, once per turn). The ChangeActive's own checks (`change_active::check_with`: the locks, the Pokémon's
/// lasting "can't retreat") come with it, on the game (execution) or legality's scratch game.
pub fn can_retreat(g: &Game, p: usize, bench_index: u8, ignore_status_conditions: bool) -> R {
    let bench = match g.st.players[p].bench.get(bench_index as usize) {
        Some(b) => *b,
        None => crate::bail!("INVALID_TARGET"),
    };
    if g.st.slot(p, bench).cards.is_empty() {
        crate::bail!("INVALID_TARGET");
    }
    let active = g.st.players[p].active;
    let sp = g.st.slot(p, active).special_conditions;
    if (sp.contains(&(SpecialCondition::Paralyzed as u8)) || sp.contains(&(SpecialCondition::Asleep as u8))) && !ignore_status_conditions {
        crate::bail!("BLOCKED_BY_SPECIAL_CONDITION");
    }
    if g.st.players[p].retreated_turn == g.st.turn {
        crate::bail!("RETREAT_ALREADY_USED");
    }
    Ok(())
}

/// The checked read of the effective Retreat Cost (`CheckRetreatCost`).
pub fn retreat_cost_read(g: &mut Game, p: usize) -> R<Cost> {
    let cost = check_retreat_cost_base(g, p);
    let (e, _) = g.run_fx(Effect::CheckRetreatCost { p: p as u8, cost, no_cost: false, reduction: 0 })?;
    Ok(match e {
        Effect::CheckRetreatCost { cost, .. } => cost,
        _ => SVec::new(),
    })
}

/// The checked reads of a retreat: the effective cost (`CheckRetreatCost`)
/// and, when the cost isn't empty, the Energy the Active Pokémon provides
/// (`CheckProvidedEnergy`; the map is empty otherwise, as no read is run).
pub fn retreat_read(g: &mut Game, p: usize) -> R<(Cost, EnergyMap)> {
    let active = g.st.players[p].active;
    let cost = retreat_cost_read(g, p)?;
    if cost.is_empty() {
        return Ok((cost, SVec::new()));
    }
    let map = crate::engine::attack::provided_energy_read(g, p, SlotRef::new(p, active))?;
    Ok((cost, map))
}

/// Can the Active Pokémon pay its Retreat Cost now? (Runs the checked reads.)
pub fn retreat_payable(g: &mut Game, p: usize) -> R<bool> {
    let (cost, map) = retreat_read(g, p)?;
    Ok(cost.is_empty() || energy::check_enough_energy(map.as_slice(), cost.as_slice()))
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    let (p, bench_index, ignore, move_to) = match *g.e(id) {
        Effect::Retreat { p, bench_index, ignore_status_conditions, move_retreat_cost_to } => {
            (p as usize, bench_index, ignore_status_conditions, move_retreat_cost_to)
        }
        _ => return Ok(()),
    };
    can_retreat(g, p, bench_index, ignore)?;
    let Some(c) = retreat_change(g, p, bench_index) else { crate::bail!("INVALID_TARGET") };
    change_active::check(g, &c)?;
    let active = g.st.players[p].active;
    let (cost, map) = retreat_read(g, p)?;
    if cost.is_empty() {
        return retreat_pokemon(g, p, bench_index);
    }
    if !energy::check_enough_energy(map.as_slice(), cost.as_slice()) {
        crate::bail!("NOT_ENOUGH_ENERGY");
    }
    if energy::check_exact_energy(map.as_slice(), cost.as_slice()) {
        let cards = energy_cards(&map);
        pay(g, p, active, &cards, move_to)?;
        return retreat_pokemon(g, p, bench_index);
    }
    if energy::all_provides_identical(map.as_slice()) {
        if let Some(sel) = energy::select_minimal_energy_for_cost(map.as_slice(), cost.as_slice()) {
            // A cost paid by Energy providing several units has a choice (ruling 1652: 1 or 2 Double Turbo Energy for cost 2).
            let has_choice = sel.len() < cost.len() && map.len() >= cost.len();
            if !sel.is_empty() && !has_choice {
                let cards: Vec<CardId> = sel.iter().map(|e| e.card).collect();
                pay(g, p, active, &cards, move_to)?;
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
    pay(g, p, active, &cards, rc.move_to)?;
    retreat_pokemon(g, p, rc.bench_index)
}

/// The Retreat Cost paid: the chosen Energy cards leave play from the Active Pokémon for their owner's discard pile (a
/// LeavePlay of attached cards by the rule, user decision D1; never refused).
fn pay(g: &mut Game, p: usize, active: crate::state::SlotId, cards: &[CardId], to: ListRef) -> R {
    let zone = crate::engine::knockout::zone_of(to);
    crate::engine::knockout::leave_play_cards_by_rule(g, SlotRef::new(p, active), cards, zone, crate::cause::Cause::rule(crate::cause::RuleWhich::Retreat, p as u8))
}

/// `new CheckRetreatCostEffect(player)`: the Active Pokémon's printed Retreat
/// Cost, plus one [C] per point of retreatCostIncreaseNextTurn (Rillaboom's
/// Drum Beating), which is part of the base cost so a "no Retreat Cost" effect
/// still wins.
pub fn check_retreat_cost_base(g: &Game, p: usize) -> Cost {
    let mut cost: Cost = SVec::new();
    if let Some(c) = g.st.active_pokemon(p) {
        for &t in g.st.cdef(c).retreat {
            cost.push(t);
        }
    }
    let a = g.st.players[p].active;
    for _ in 0..g.st.slot(p, a).retreat_cost_increase_next_turn.max(0) {
        cost.push(ct::COLORLESS);
    }
    cost
}
