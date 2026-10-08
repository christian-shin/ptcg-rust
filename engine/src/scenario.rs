//! Scenarios: the board edits of the oracle's `scenario.ts`, applied at the
//! same point of a replay (the first turn decision on or after
//! `scenario.turn`) with the same primitives, so a trace that starts from a
//! crafted position replays like any other. Keep this file and scenario.ts
//! identical; the format is documented there and in CARD_PORTING.md.
//! Scripted `answers` need nothing here: the trace records every answer.

use crate::engine::game_effect::reset_empty_slot;
use crate::engine::phase::add_condition;
use crate::engine::turn::switch_pokemon;
use crate::game::Game;
use crate::list::{CardId, CardList};
use crate::state::{ListRef, SlotId};
use crate::types::SpecialCondition;
use serde_json::Value;

/// `scenarioTurn`: the turn the edits wait for (default 2).
pub fn scenario_turn(sc: &Value) -> i32 {
    sc["turn"].as_i64().unwrap_or(2) as i32
}

/// Deck first (top down), else hand.
fn take(g: &Game, p: usize, name: &str) -> Result<(ListRef, CardId), String> {
    for from in [ListRef::Deck(p as u8), ListRef::Hand(p as u8)] {
        if let Some(&c) = g.lst(from).iter().find(|&&c| crate::carddb::card_is(g.st.cdef(c), name)) {
            return Ok((from, c));
        }
    }
    Err(format!("scenario: no {} in deck or hand", name))
}

fn mv(g: &mut Game, p: usize, name: &str, dst: ListRef) -> Result<CardId, String> {
    let (from, c) = take(g, p, name)?;
    g.move_card_to(from, c, dst);
    Ok(c)
}

fn empty_bench(g: &Game, p: usize) -> Result<SlotId, String> {
    let bench = &g.st.players[p].bench;
    bench.iter().copied().find(|&s| g.lst(ListRef::Slot(p as u8, s)).is_empty()).ok_or_else(|| "scenario: no empty Bench spot".to_string())
}

fn names(v: &Value) -> Vec<&str> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default()
}

/// A Pokémon or an evolution stack (Basic first).
fn stack(v: &Value) -> Vec<&str> {
    match v.as_str() {
        Some(s) => vec![s],
        None => names(v),
    }
}

fn condition(c: &str) -> Result<SpecialCondition, String> {
    Ok(match c {
        "PARALYZED" => SpecialCondition::Paralyzed,
        "CONFUSED" => SpecialCondition::Confused,
        "ASLEEP" => SpecialCondition::Asleep,
        "POISONED" => SpecialCondition::Poisoned,
        "BURNED" => SpecialCondition::Burned,
        _ => return Err(format!("scenario: unknown condition {}", c)),
    })
}

/// `dress`: Energy, Tool, damage, conditions and played turn.
#[allow(clippy::too_many_arguments)]
fn dress(g: &mut Game, p: usize, s: SlotId, energy: &Value, tool: &Value, damage: &Value, conditions: &Value, played: &Value) -> Result<(), String> {
    let pu = p as u8;
    for n in names(energy) {
        mv(g, p, n, ListRef::Slot(pu, s))?;
    }
    for n in stack(tool) {
        let c = mv(g, p, n, ListRef::Slot(pu, s))?;
        let slot = &mut g.st.players[p].slots[s as usize];
        slot.cards.remove(c);
        slot.tools.push(c);
    }
    if let Some(d) = damage.as_i64() {
        g.st.players[p].slots[s as usize].damage = d as i32;
    }
    for c in names(conditions) {
        add_condition(&mut g.st.players[p].slots[s as usize], condition(c)?);
    }
    let turn = g.st.turn;
    g.st.players[p].slots[s as usize].pokemon_played_turn = if played.as_str() == Some("this_turn") { turn } else { 0 };
    Ok(())
}

/// `resetPlayer`: everything back to the deck, every slot emptied (bench first, then Active).
fn reset_player(g: &mut Game, p: usize) {
    let pu = p as u8;
    let deck = ListRef::Deck(pu);
    g.move_to(ListRef::Hand(pu), deck, None);
    g.move_to(ListRef::Discard(pu), deck, None);
    for i in 0..6 {
        g.move_to(ListRef::Prize(pu, i), deck, None);
    }
    // A fresh board: Prize cards a card effect turned face up before the scenario turn are face down again.
    g.st.players[p].prize_public = [false; 6];
    g.st.players[p].prize_face_up = [false; 6];
    g.move_to(ListRef::Stadium(pu), deck, None);
    let mut slots: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
    slots.push(g.st.players[p].active);
    for s in slots {
        let slot = &g.st.players[p].slots[s as usize];
        let cards: Vec<CardId> = slot.cards.iter().chain(slot.tools.iter()).collect();
        for c in cards {
            g.move_card_to(ListRef::Slot(pu, s), c, deck);
        }
        reset_empty_slot(&mut g.st.players[p].slots[s as usize]);
    }
}

fn apply_side(g: &mut Game, p: usize, side: &Value) -> Result<(), String> {
    let pu = p as u8;
    let reset = side["reset"].as_bool() == Some(true);
    if side["hand_to_deck"].as_bool() == Some(true) {
        g.move_to(ListRef::Hand(pu), ListRef::Deck(pu), None);
    }
    for n in names(&side["discard"]) {
        mv(g, p, n, ListRef::Discard(pu))?;
    }
    for n in names(&side["hand"]) {
        mv(g, p, n, ListRef::Hand(pu))?;
    }
    for n in names(&side["deck_top"]).into_iter().rev() {
        let (from, c) = take(g, p, n)?;
        let i = g.lst(from).iter().position(|&x| x == c).unwrap();
        g.lst_mut(from).remove_at(i);
        g.lst_mut(ListRef::Deck(pu)).insert(0, c);
    }
    for (i, n) in names(&side["prizes"]).into_iter().enumerate() {
        if i >= 6 {
            return Err(format!("scenario: no Prize {}", i));
        }
        g.move_to(ListRef::Prize(pu, i as u8), ListRef::Deck(pu), None);
        mv(g, p, n, ListRef::Prize(pu, i as u8))?;
    }
    if let Some(n) = side["stadium"].as_str() {
        if (0..2).any(|q| !g.lst(ListRef::Stadium(q)).is_empty()) {
            return Err("scenario: a Stadium is already in play".into());
        }
        mv(g, p, n, ListRef::Stadium(pu))?;
    }
    let active = g.st.players[p].active;
    if !side["active"].is_null() && g.lst(ListRef::Slot(pu, active)).is_empty() {
        // After `reset`: the Active Spot is empty, so fill it directly.
        for n in stack(&side["active"]) {
            mv(g, p, n, ListRef::Slot(pu, active))?;
        }
        dress(g, p, active, &side["active_energy"], &side["active_tool"], &side["active_damage"], &side["active_conditions"], &side["active_played"])?;
    } else if !side["active"].is_null() {
        let s = empty_bench(g, p)?;
        for n in stack(&side["active"]) {
            mv(g, p, n, ListRef::Slot(pu, s))?;
        }
        switch_pokemon(g, p, s).map_err(|e| e.0.to_string())?;
        let a = g.st.players[p].active;
        dress(g, p, a, &side["active_energy"], &side["active_tool"], &side["active_damage"], &side["active_conditions"], &side["active_played"])?;
    } else {
        let a = g.st.players[p].active;
        for n in names(&side["active_energy"]) {
            mv(g, p, n, ListRef::Slot(pu, a))?;
        }
        if let Some(d) = side["active_damage"].as_i64() {
            g.st.players[p].slots[a as usize].damage = d as i32;
        }
        for c in names(&side["active_conditions"]) {
            add_condition(&mut g.st.players[p].slots[a as usize], condition(c)?);
        }
    }
    for b in side["bench"].as_array().into_iter().flatten() {
        let s = empty_bench(g, p)?;
        for n in stack(&b["card"]) {
            mv(g, p, n, ListRef::Slot(pu, s))?;
        }
        dress(g, p, s, &b["energy"], &b["tool"], &b["damage"], &b["conditions"], &b["played"])?;
    }
    let turn = g.st.turn;
    let pl = &mut g.st.players[p];
    if side["supporter_played"].as_bool() == Some(true) {
        pl.supporter_turn = turn;
    }
    if side["energy_attached"].as_bool() == Some(true) {
        pl.energy_played_turn = turn;
    }
    if side["stadium_played"].as_bool() == Some(true) {
        pl.stadium_played_turn = turn;
    }
    if side["retreated"].as_bool() == Some(true) {
        pl.retreated_turn = turn;
    }
    if reset {
        // Prizes not named are refilled from the top of the deck, last.
        for i in 0..6 {
            if g.lst(ListRef::Prize(pu, i)).is_empty() {
                g.move_to(ListRef::Deck(pu), ListRef::Prize(pu, i), Some(1));
            }
        }
    }
    if let Some(n) = side["prizes_left"].as_u64() {
        if n > 6 {
            return Err("scenario: prizes_left out of range".into());
        }
        for i in n as u8..6 {
            g.move_to(ListRef::Prize(pu, i), ListRef::Deck(pu), None);
        }
    }
    if let Some(n) = side["deck_left"].as_u64() {
        while g.st.players[p].deck.len() > n as usize {
            g.move_to(ListRef::Deck(pu), ListRef::Discard(pu), Some(1));
        }
    }
    Ok(())
}

/// `applyScenario`: `me` is the player whose turn it is.
pub fn apply(g: &mut Game, sc: &Value) -> Result<(), String> {
    let me = g.st.active_player as usize;
    // Resets first (both players), so later edits see the cleared board.
    for (p, side) in [(me, &sc["me"]), (1 - me, &sc["opp"])] {
        if side["reset"].as_bool() == Some(true) {
            if side["active"].is_null() {
                return Err("scenario: reset needs an active Pokemon".into());
            }
            reset_player(g, p);
        }
    }
    apply_side(g, me, &sc["me"])?;
    apply_side(g, 1 - me, &sc["opp"])?;
    let coins: Vec<bool> = sc["coins"].as_array().map(|a| a.iter().filter_map(|x| x.as_bool()).collect()).unwrap_or_default();
    if sc["sudden_death"].as_bool() == Some(true) {
        g.st.is_sudden_death = true;
    }
    g.rng.force_coins(&coins);
    Ok(())
}
