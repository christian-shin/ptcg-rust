//! Turn-level options: structural candidates filtered by trial dispatch on a
//! clone (cheap: the whole game is `Copy`). Mirrors the oracle's
//! `turnCandidates` + `legalTurnOptions` exactly, including order.

use crate::effects::*;
use crate::engine::turn::check_attacks_effect;
use crate::game::{Action, Game};
use crate::list::*;
use crate::prompts::{slot_targets, target_json};
use crate::types::*;
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct TurnOption {
    pub desc: Value,
    pub action: Action,
}

const BOARD: CardTarget = CardTarget::new(PlayerType::BottomPlayer, SlotType::Board, 0);

/// JavaScript default sort order for strings (UTF-16 code units).
fn js_sort(v: &mut Vec<&'static str>) {
    v.sort_by(|a, b| {
        let x: Vec<u16> = a.encode_utf16().collect();
        let y: Vec<u16> = b.encode_utf16().collect();
        x.cmp(&y)
    });
}

/// Canonical descriptor of a turn action (oracle `TurnOption.desc`).
pub fn describe_action(g: &Game, a: Action) -> Value {
    let p = g.st.active_player as usize;
    match a {
        Action::PlayCard { hand_index, target } => {
            let c = g.st.players[p].hand.as_slice()[hand_index as usize];
            json!({ "a": "play", "card": g.card_ref(c), "target": target_json(target) })
        }
        Action::Attack { name } => json!({ "a": "attack", "name": name }),
        Action::UseAbility { name, target } => json!({ "a": "ability", "name": name, "source": target_json(target) }),
        Action::UseTrainerAbility { name, target } => json!({ "a": "trainerAbility", "name": name, "source": target_json(target) }),
        Action::UseStadium => json!({ "a": "stadium" }),
        Action::Retreat { bench_index } => json!({ "a": "retreat", "bench": bench_index }),
        Action::Pass => json!({ "a": "pass" }),
    }
}

pub fn turn_candidates(g: &Game) -> Vec<TurnOption> {
    candidate_actions(g).into_iter().map(|a| TurnOption { desc: describe_action(g, a), action: a }).collect()
}

/// Structural candidates for the active player's turn, in oracle order.
pub fn candidate_actions(g: &Game) -> Vec<Action> {
    let mut out = Vec::with_capacity(32);
    let p = g.st.active_player as usize;
    let own = slot_targets(&g.st, p, PlayerType::BottomPlayer, &[SlotType::Active as u8, SlotType::Bench as u8]);
    let pl = &g.st.players[p];
    let first_empty = pl.bench.iter().position(|b| pl.slots[*b as usize].cards.is_empty());
    for (hi, c) in pl.hand.iter().enumerate() {
        let d = g.st.cdef(c);
        let hand_index = hi as u8;
        if d.is_energy() {
            for t in &own {
                out.push(Action::PlayCard { hand_index, target: *t });
            }
        } else if d.is_pokemon() {
            if let Some(i) = first_empty {
                out.push(Action::PlayCard { hand_index, target: CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8) });
            }
            if d.stage != Stage::Basic as u8 {
                for t in &own {
                    out.push(Action::PlayCard { hand_index, target: *t });
                }
            }
        } else if d.is_trainer() {
            if d.trainer_type == TrainerType::Tool as u8 {
                for t in &own {
                    out.push(Action::PlayCard { hand_index, target: *t });
                }
            } else {
                out.push(Action::PlayCard { hand_index, target: BOARD });
            }
        }
    }

    let mut names: Vec<&'static str> = Vec::new();
    let add = |n: &'static str, names: &mut Vec<&'static str>| {
        if !names.contains(&n) {
            names.push(n);
        }
    };
    if let Some(c) = g.st.active_pokemon(p) {
        for a in g.st.cdef(c).attacks {
            add(a.name, &mut names);
        }
    }
    for &b in pl.bench.iter() {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            for a in g.st.cdef(c).attacks.iter().filter(|a| a.use_on_bench) {
                add(a.name, &mut names);
            }
        }
    }
    if g.kinds_present & (1u128 << crate::effects::k::CHECK_POKEMON_ATTACKS) != 0 || g.st.slot(p, pl.active).tools.len() > 0 {
        let mut sim = *g;
        if let Ok((Effect::CheckPokemonAttacks { attacks, .. }, _)) = sim.run_fx(check_attacks_effect(&sim, p)) {
            for a in attacks.iter() {
                add(g.st.cdef(a.card).attacks[a.idx()].name, &mut names);
            }
        }
    }
    js_sort(&mut names);
    for n in names {
        out.push(Action::Attack { name: n });
    }

    for t in &own {
        let slot = crate::prompts::get_target(&g.st, p, *t).unwrap();
        if let Some(c) = g.st.slot_pokemon(slot.p as usize, slot.s) {
            let mut pn: Vec<&'static str> = Vec::new();
            for pw in g.st.cdef(c).powers {
                if !pn.contains(&pw.name) {
                    pn.push(pw.name);
                }
            }
            if g.kinds_present & (1u128 << crate::effects::k::CHECK_POKEMON_POWERS) != 0 {
                let mut sim = *g;
                let mut powers = SVec::new();
                for i in 0..g.st.cdef(c).powers.len() {
                    powers.push(PowerRef { card: c, index: i as u8 });
                }
                if let Ok((Effect::CheckPokemonPowers { powers, .. }, _)) = sim.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: c, powers }) {
                    for r in powers.iter() {
                        let n = g.st.cdef(r.card).powers[r.index as usize].name;
                        if !pn.contains(&n) {
                            pn.push(n);
                        }
                    }
                }
            }
            js_sort(&mut pn);
            for n in pn {
                out.push(Action::UseAbility { name: n, target: *t });
            }
        }
    }
    if g.st.stadium_card().is_some() {
        out.push(Action::UseStadium);
    }
    for (i, b) in pl.bench.iter().enumerate() {
        if !pl.slots[*b as usize].cards.is_empty() {
            out.push(Action::Retreat { bench_index: i as u8 });
        }
    }
    out.push(Action::Pass);
    out
}

/// Legal turn options (deduplicated, in candidate order).
pub fn legal_turn_options(g: &Game) -> Vec<TurnOption> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for c in turn_candidates(g) {
        let key = serde_json::to_string(&c.desc).unwrap();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        if is_legal(g, c.action) {
            out.push(c);
        }
    }
    out
}

/// Legality for a single action without building descriptors (fast path).
pub fn is_legal(g: &Game, a: Action) -> bool {
    let mut trial = *g;
    if trial.act_trial(a).is_err() {
        return false;
    }
    // Resolve info prompts (an ability's animation wait) so checks that run
    // after them count toward legality; stop at chance prompts and decisions.
    // Like Twinleaf, a coin is drawn when its CoinFlipEffect resolves and the
    // callback runs after the "Coin flip animation" wait, so a callback that
    // throws makes legality depend on the trial's draw (Rust: a copy of the
    // game RNG; oracle: its separate simulation stream).
    for _ in 0..100 {
        if trial.st.phase == crate::types::GamePhase::Finished {
            break;
        }
        match trial.pending() {
            crate::game::Pending::Info(i) => {
                if trial.resolve(i, crate::prompts::Res::True).is_err() {
                    return false;
                }
            }
            _ => break,
        }
    }
    true
}

/// Legal actions without descriptors (fast path for the select interface).
pub fn legal_actions(g: &Game) -> Vec<TurnOption> {
    let mut out: Vec<TurnOption> = Vec::new();
    for c in turn_candidates_fast(g) {
        if out.iter().any(|o| o.action == c) {
            continue;
        }
        if is_legal(g, c) {
            out.push(TurnOption { desc: Value::Null, action: c });
        }
    }
    out
}

fn turn_candidates_fast(g: &Game) -> Vec<Action> {
    candidate_actions(g)
}
