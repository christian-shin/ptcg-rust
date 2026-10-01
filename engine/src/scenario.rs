//! Scenarios: the board edits of the oracle's `scenario.ts`, applied at the
//! same point of a replay (the first turn decision on or after
//! `scenario.turn`) with the same primitives, so a trace that starts from a
//! crafted position replays like any other. See scenario.ts for the format.

use crate::engine::turn::switch_pokemon;
use crate::game::Game;
use crate::list::CardId;
use crate::state::{ListRef, SlotId};
use serde_json::Value;

/// `scenarioTurn`: the turn the edits wait for (default 2).
pub fn scenario_turn(sc: &Value) -> i32 {
    sc["turn"].as_i64().unwrap_or(2) as i32
}

/// Deck first (top down), else hand.
fn take(g: &Game, p: usize, name: &str) -> Result<(ListRef, CardId), String> {
    for from in [ListRef::Deck(p as u8), ListRef::Hand(p as u8)] {
        if let Some(&c) = g.lst(from).iter().find(|&&c| g.st.cdef(c).full_name == name) {
            return Ok((from, c));
        }
    }
    Err(format!("scenario: no {} in deck or hand", name))
}

fn mv(g: &mut Game, p: usize, name: &str, dst: ListRef) -> Result<(), String> {
    let (from, c) = take(g, p, name)?;
    g.move_card_to(from, c, dst);
    Ok(())
}

fn empty_bench(g: &Game, p: usize) -> Result<SlotId, String> {
    let bench = &g.st.players[p].bench;
    bench.iter().copied().find(|&s| g.lst(ListRef::Slot(p as u8, s)).is_empty()).ok_or_else(|| "scenario: no empty Bench spot".to_string())
}

fn names(v: &Value) -> Vec<&str> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default()
}

fn apply_side(g: &mut Game, p: usize, side: &Value) -> Result<(), String> {
    let pu = p as u8;
    if side["hand_to_deck"].as_bool() == Some(true) {
        g.move_to(ListRef::Hand(pu), ListRef::Deck(pu), None);
    }
    for n in names(&side["discard"]) {
        mv(g, p, n, ListRef::Discard(pu))?;
    }
    for n in names(&side["hand"]) {
        mv(g, p, n, ListRef::Hand(pu))?;
    }
    if let Some(n) = side["active"].as_str() {
        let s = empty_bench(g, p)?;
        mv(g, p, n, ListRef::Slot(pu, s))?;
        switch_pokemon(g, p, s).map_err(|e| e.0.to_string())?;
    }
    for n in names(&side["active_energy"]) {
        let a = g.st.players[p].active;
        mv(g, p, n, ListRef::Slot(pu, a))?;
    }
    if let Some(d) = side["active_damage"].as_i64() {
        let a = g.st.players[p].active;
        g.st.players[p].slots[a as usize].damage = d as i32;
    }
    for b in side["bench"].as_array().into_iter().flatten() {
        let s = empty_bench(g, p)?;
        mv(g, p, b["card"].as_str().unwrap_or(""), ListRef::Slot(pu, s))?;
        for n in names(&b["energy"]) {
            mv(g, p, n, ListRef::Slot(pu, s))?;
        }
        if let Some(d) = b["damage"].as_i64() {
            g.st.players[p].slots[s as usize].damage = d as i32;
        }
    }
    Ok(())
}

/// `applyScenario`: `me` is the player whose turn it is.
pub fn apply(g: &mut Game, sc: &Value) -> Result<(), String> {
    let me = g.st.active_player as usize;
    apply_side(g, me, &sc["me"])?;
    apply_side(g, 1 - me, &sc["opp"])
}
