//! Replay oracle traces through the Rust engine and stop at the first
//! divergence.
//!
//!   diff <trace.json|dir>... [--dump <dir>] [--quiet] [--strict]
//!
//! Traces that record observable hashes (`o`) are replayed for observable
//! parity (`replay_obs`); `--strict` or PTCG_OBS=0 forces the
//! lockstep state-hash replay below, which older traces always get.
//!
//! A scenario's `expect` assertions (docs/ENGINE.md, "Scenarios and expect"; local) are checked
//! here against the Rust state and reported as their own outcome, EXPECT FAILED,
//! apart from divergence from the oracle.
//!
//! For each trace: rebuild the game from seed and decks, then at every step
//! compare the decision descriptor / turn option set, apply the recorded
//! answer, settle chance and info prompts, and compare the state hash.
//! With `--dump`, the Rust canonical state at the diverging step is written
//! as `<dir>/<trace>.rust.json` for a structural diff against the oracle.

use ptcg::carddb::def_by_full_name;
use ptcg::game::{Game, Pending};
use ptcg::options::legal_turn_options;
use serde_json::Value;
use std::path::{Path, PathBuf};

thread_local! {
    /// Step being replayed, reported when a panic aborts the replay.
    static PANIC_STEP: std::cell::Cell<isize> = const { std::cell::Cell::new(-1) };
}

/// invariants at every turn decision (`PTCG_NO_INVARIANTS=1` turns them off).
fn check_invariants() -> bool {
    std::env::var("PTCG_NO_INVARIANTS").map_or(true, |v| v.is_empty() || v == "0")
}

#[derive(Debug)]
enum Outcome {
    Pass { steps: usize },
    Diverged { step: isize, what: String, detail: String },
    Unsupported(String),
    /// The replay matched the oracle but the scenario's `expect` assertions failed (or are malformed).
    ExpectFailed { failures: Vec<String> },
}

fn canon(v: &Value) -> String {
    serde_json::to_string(v).unwrap()
}

fn sorted_set(v: &[Value]) -> Vec<String> {
    let mut s: Vec<String> = v.iter().map(canon).collect();
    s.sort();
    s
}

/// The card's key ("Name SET NUM").
fn card_label(d: u16) -> String {
    ptcg::carddb::en_key(d).to_string()
}

fn replay(trace: &Value, dump: Option<&Path>, name: &str) -> Outcome {
    let header = &trace["header"];
    let seed = header["seed"].as_u64().unwrap() as u32;
    let mut decks: [Vec<u16>; 2] = [Vec::new(), Vec::new()];
    for p in 0..2 {
        for n in header["decks"][p].as_array().unwrap() {
            let n = n.as_str().unwrap();
            match def_by_full_name(n) {
                Some(d) => decks[p].push(d),
                None => return Outcome::Unsupported(format!("unknown card {}", n)),
            }
        }
    }
    for d in decks.iter().flatten() {
        if ptcg::cards::missing_behavior(*d) {
            return Outcome::Unsupported(format!("card not ported: {}", card_label(*d)));
        }
    }
    let mut g = Game::new(seed);
    let dump_state = |g: &Game, step: isize| {
        if let Some(dir) = dump {
            let path = dir.join(format!("{}.step{}.rust.json", name, step));
            let _ = std::fs::write(&path, serde_json::to_string_pretty(&g.canonical_json()).unwrap());
        }
    };
    if let Err(e) = g.start([&decks[0], &decks[1]]).and_then(|_| g.settle()) {
        return Outcome::Diverged { step: -1, what: "error".into(), detail: format!("{:?}", e) };
    }
    let start_h = trace["start"]["h"].as_str().unwrap();
    if g.state_hash() != start_h {
        dump_state(&g, -1);
        return Outcome::Diverged { step: -1, what: "hash".into(), detail: "start state".into() };
    }
    if let Some(o) = trace["start"]["o"].as_str() {
        if g.observable_hash() != o {
            return Outcome::Diverged { step: -1, what: "obs-hash".into(), detail: "start state: projections disagree".into() };
        }
    }
    let steps = trace["steps"].as_array().unwrap();
    // Scenario edits (oracle scenario.ts): applied at the first turn decision
    // on or after `scenario.turn`, then checked against the recorded hash.
    let scenario = &header["scenario"];
    let mut scenario_done = scenario.is_null();
    for (i, st) in steps.iter().enumerate() {
        PANIC_STEP.with(|c| c.set(i as isize));
        let d = &st["d"];
        let a = &st["a"];
        let r = match g.pending() {
            Pending::Decision(pi) => {
                if d["kind"] != "prompt" {
                    return Outcome::Diverged { step: i as isize, what: "kind".into(), detail: format!("rust prompt, oracle {}", d["kind"]) };
                }
                let pr = g.prompts.as_slice()[pi];
                let rd = g.describe_prompt(&pr);
                if canon(&rd) != canon(d) {
                    dump_state(&g, i as isize);
                    return Outcome::Diverged {
                        step: i as isize,
                        what: "prompt".into(),
                        detail: format!("rust {}\noracle {}", canon(&rd), canon(d)),
                    };
                }
                match g.decode_answer(&pr, a) {
                    Ok(res) => g.resolve(pi, res),
                    Err(e) => {
                        return Outcome::Diverged { step: i as isize, what: "decode".into(), detail: format!("{:?} for {}", e, canon(a)) }
                    }
                }
            }
            Pending::Turn(_) => {
                ptcg::expect::on_turn_decision(&g);
                if !scenario_done && g.st.turn >= ptcg::scenario::scenario_turn(scenario) {
                    scenario_done = true;
                    if let Err(e) = ptcg::scenario::apply(&mut g, scenario) {
                        return Outcome::Diverged { step: i as isize, what: "scenario".into(), detail: e };
                    }
                    match ptcg::expect::parse(scenario) {
                        Ok(a) if !a.is_empty() => {
                            ptcg::expect::arm(&g, a, dump.is_some());
                            ptcg::expect::on_scenario_start(&g);
                        }
                        Ok(_) => {}
                        Err(e) => return Outcome::ExpectFailed { failures: vec![format!("invalid expect: {}", e)] },
                    }
                    let at = &trace["scenario"];
                    if at["step"].as_u64() != Some(i as u64) || at["h"].as_str() != Some(g.state_hash().as_str()) {
                        dump_state(&g, i as isize);
                        return Outcome::Diverged { step: i as isize, what: "scenario".into(), detail: format!("oracle applied at {}", at) };
                    }
                }
                if d["kind"] != "turn" {
                    return Outcome::Diverged { step: i as isize, what: "kind".into(), detail: format!("rust turn, oracle {}", canon(d)) };
                }
                let opts = legal_turn_options(&g);
                let rust: Vec<Value> = opts.iter().map(|o| o.desc.clone()).collect();
                let oracle = d["options"].as_array().unwrap();
                if sorted_set(&rust) != sorted_set(oracle) {
                    let rs = sorted_set(&rust);
                    let os = sorted_set(oracle);
                    let only_r: Vec<&String> = rs.iter().filter(|x| !os.contains(x)).collect();
                    let only_o: Vec<&String> = os.iter().filter(|x| !rs.contains(x)).collect();
                    dump_state(&g, i as isize);
                    return Outcome::Diverged {
                        step: i as isize,
                        what: "options".into(),
                        detail: format!("only rust {:?}\nonly oracle {:?}", only_r, only_o),
                    };
                }
                let want = canon(a);
                let k = opts.iter().position(|o| canon(&o.desc) == want).unwrap();
                g.act(opts[k].action)
            }
            other => {
                return Outcome::Diverged { step: i as isize, what: "pending".into(), detail: format!("rust {:?}, oracle {}", other, d["kind"]) }
            }
        };
        if let Err(e) = r.and_then(|_| g.settle()) {
            // The oracle hit the same GameError on its last step (e.g. an
            // ability whose checks only run after the in-play WaitPrompt):
            // the trace ends there, so compare the state left behind.
            let res = &trace["result"];
            let same_error = i + 1 == steps.len() && res["status"] == "error" && res["message"].as_str() == Some(e.0);
            if same_error && g.state_hash() == st["h"].as_str().unwrap_or("") {
                return Outcome::Pass { steps: steps.len() };
            }
            dump_state(&g, i as isize);
            // The oracle run also stopped on this GameError (e.g. an ability
            // offered as legal whose PowerEffect throws after the animation
            // wait): parity holds if the message and resulting state match.
            let res = &trace["result"];
            let same_error = i + 1 == steps.len() && res["status"] == "error" && res["message"].as_str() == Some(e.0);
            if same_error && st["h"].as_str() == Some(g.state_hash().as_str()) {
                return Outcome::Pass { steps: steps.len() };
            }
            return Outcome::Diverged { step: i as isize, what: "error".into(), detail: format!("{:?}", e) };
        }
        let h = st["h"].as_str().unwrap();
        if g.state_hash() != h {
            dump_state(&g, i as isize);
            return Outcome::Diverged { step: i as isize, what: "hash".into(), detail: String::new() };
        }
        // Same canonical state, so the two projections must agree (checks
        // `observable_json` against the oracle's `observableState`).
        if let Some(o) = st["o"].as_str() {
            if g.observable_hash() != o {
                return Outcome::Diverged { step: i as isize, what: "obs-hash".into(), detail: "projections disagree".into() };
            }
        }
        if check_invariants() && matches!(g.pending(), Pending::Turn(_)) {
            if let Err(e) = ptcg::invariants::check(&g) {
                dump_state(&g, i as isize);
                return Outcome::Diverged { step: i as isize, what: "invariant".into(), detail: e };
            }
        }
    }
    Outcome::Pass { steps: steps.len() }
}

/// A prompt descriptor without plumbing: no message names or
/// filters, a card choice reduced to the cards that can really be chosen
/// (`selectable` minus `options.blocked`), and no per-kind caps that `max`
/// already implies. Select prompts keep only how many values they offer.
fn norm_prompt(d: &Value) -> Value {
    let mut o = d.as_object().cloned().unwrap_or_default();
    o.remove("message");
    o.remove("filter");
    o.remove("kind");
    let blocked: Vec<u64> = o.get("options").and_then(|x| x.get("blocked")).and_then(|b| b.as_array()).map(|b| b.iter().filter_map(|v| v.as_u64()).collect()).unwrap_or_default();
    if let Some(opts) = o.get_mut("options").and_then(|x| x.as_object_mut()) {
        opts.remove("blocked");
        // A per-kind cap (`maxPokemons`, ...) no smaller than `max` can't
        // constrain the choice.
        if let Some(max) = opts.get("max").and_then(|m| m.as_i64()) {
            opts.retain(|k, v| !(k.starts_with("max") && k != "max" && v.as_i64().is_some_and(|c| c >= max)));
        }
    }
    if let (Some(cards), Some(sel)) = (o.get("cards").and_then(|c| c.as_array()).cloned(), o.get("selectable").and_then(|c| c.as_array()).cloned()) {
        let pick: Vec<Value> = cards
            .iter()
            .enumerate()
            .filter(|(i, _)| sel.get(*i).and_then(|v| v.as_bool()).unwrap_or(false) && !blocked.contains(&(*i as u64)))
            .map(|(_, c)| c.clone())
            .collect();
        o.remove("selectable");
        o.insert("selectableCards".into(), Value::Array(pick));
    }
    if let Some(v) = o.get("values").and_then(|v| v.as_array()).map(|v| v.len()) {
        o.insert("values".into(), Value::from(v));
    }
    Value::Object(o)
}

/// A normalized prompt with its card lists as sets (sorted).
fn card_set(d: &Value) -> Value {
    let mut o = d.clone();
    for k in ["cards", "selectableCards"] {
        if let Some(a) = o.get_mut(k).and_then(|a| a.as_array_mut()) {
            a.sort_by_key(|c| c.to_string());
        }
    }
    o
}

/// An answer given against the card list `from`, rewritten for the same cards listed as `to`:
/// card indices (`[i, ...]`, or `{"index": i, ...}` entries) follow their card.
fn remap_answer(a: &Value, from: &Value, to: &Value) -> Value {
    let (Some(from), Some(to)) = (from.as_array(), to.as_array()) else { return a.clone() };
    let map = |i: u64| -> Value {
        let pos = from.get(i as usize).and_then(|c| to.iter().position(|t| t == c));
        Value::from(pos.map_or(i, |p| p as u64))
    };
    let Some(arr) = a.as_array() else { return a.clone() };
    Value::Array(
        arr.iter()
            .map(|v| match v {
                Value::Number(n) => n.as_u64().map_or(v.clone(), map),
                Value::Object(o) => {
                    let mut o = o.clone();
                    if let Some(i) = o.get("index").and_then(|x| x.as_u64()) {
                        o.insert("index".into(), map(i));
                    }
                    Value::Object(o)
                }
                _ => v.clone(),
            })
            .collect(),
    )
}

/// The oracle's recorded chance outcomes as a replay tape (ptcg::rng).
fn tape_of(events: &[Value]) -> Vec<ptcg::rng::Draw> {
    use ptcg::rng::Draw;
    events
        .iter()
        .filter_map(|c| match c["k"].as_str()? {
            "coin" => Some(Draw::Coin(c["v"].as_bool()?)),
            "shuffle" => Some(Draw::Shuffle(c["v"].as_array()?.iter().filter_map(|x| x.as_u64().map(|x| x as u8)).collect())),
            "index" => Some(Draw::Index(c["n"].as_u64()? as usize, c["v"].as_u64()? as usize)),
            _ => None,
        })
        .collect()
}

/// Observable-parity replay (`--obs`). The oracle's turn
/// decisions split the trace into segments. At each turn decision Rust must
/// reach the same player-observable state (`o`) and offer the same turn
/// options. Inside a segment Rust's prompts are answered by the oracle prompt
/// with the same player and prompt class (first unused one), whose plumbing-free
/// descriptor must match; oracle prompts Rust never asks are allowed (a "you
/// may" with no effect, say) because the next observable state still has to
/// match. Chance comes from the segment's recorded outcomes by kind, through
/// the RNG replay tape, so card code that draws directly is covered too.
fn replay_obs(trace: &Value, dump: Option<&Path>, name: &str) -> Outcome {
    let header = &trace["header"];
    let seed = header["seed"].as_u64().unwrap() as u32;
    let mut decks: [Vec<u16>; 2] = [Vec::new(), Vec::new()];
    for p in 0..2 {
        for n in header["decks"][p].as_array().unwrap() {
            let n = n.as_str().unwrap();
            match def_by_full_name(n) {
                Some(d) => decks[p].push(d),
                None => return Outcome::Unsupported(format!("unknown card {}", n)),
            }
        }
    }
    for d in decks.iter().flatten() {
        if ptcg::cards::missing_behavior(*d) {
            return Outcome::Unsupported(format!("card not ported: {}", card_label(*d)));
        }
    }
    if trace["start"]["o"].as_str().is_none() {
        return Outcome::Unsupported("trace has no observable hashes (regenerate it)".into());
    }
    let steps = trace["steps"].as_array().unwrap();
    let dump_state = |g: &Game, step: isize| {
        if let Some(dir) = dump {
            let path = dir.join(format!("{}.step{}.rust.json", name, step));
            let _ = std::fs::write(&path, serde_json::to_string_pretty(&ptcg::canonical::observable_json(&g.canonical_json())).unwrap());
        }
    };
    // Segment boundaries: indices of the oracle's turn steps.
    let turns: Vec<usize> = steps.iter().enumerate().filter(|(_, s)| s["d"]["kind"] == "turn").map(|(i, _)| i).collect();
    let segment = |from: usize| -> (Vec<usize>, Vec<Value>) {
        let end = turns.iter().copied().find(|&t| t > from).unwrap_or(steps.len());
        let prompts: Vec<usize> = (from..end).filter(|&i| steps[i]["d"]["kind"] == "prompt").collect();
        let pool: Vec<Value> = (from..end).flat_map(|i| steps[i]["c"].as_array().cloned().unwrap_or_default()).collect();
        (prompts, pool)
    };
    // Observable hash the oracle reached before step `i`.
    let o_before = |i: usize| -> &str { if i == 0 { trace["start"]["o"].as_str().unwrap() } else { steps[i - 1]["o"].as_str().unwrap_or("") } };

    let mut g = Game::new(seed);
    let (mut queue, mut pool) = segment(0);
    pool.splice(0..0, trace["start"]["c"].as_array().cloned().unwrap_or_default());
    ptcg::rng::set_tape(Some(tape_of(&pool)));
    if let Err(e) = g.start([&decks[0], &decks[1]]).and_then(|_| g.settle()) {
        return Outcome::Diverged { step: -1, what: "error".into(), detail: format!("{:?}", e) };
    }
    let scenario = &header["scenario"];
    let mut scenario_done = scenario.is_null();
    let mut next_turn = 0usize; // index into `turns`
    let mut decisions = 0usize;
    let finish = |g: &Game, decisions: usize| -> Outcome {
        let want = steps.last().and_then(|s| s["o"].as_str()).unwrap_or(trace["start"]["o"].as_str().unwrap());
        if g.observable_hash() != want {
            dump_state(g, steps.len() as isize);
            return Outcome::Diverged { step: steps.len() as isize, what: "obs".into(), detail: "final state".into() };
        }
        Outcome::Pass { steps: decisions }
    };
    loop {
        decisions += 1;
        if decisions > 20_000 {
            return Outcome::Diverged { step: -1, what: "loop".into(), detail: "too many decisions".into() };
        }
        let r = match g.pending() {
            Pending::Decision(pi) => {
                let pr = g.prompts.as_slice()[pi];
                let rd = g.describe_prompt(&pr);
                // Same player and class; one with the same descriptor first, so an oracle prompt Rust
                // doesn't ask (a search of an empty deck, with nothing to choose) is skipped rather
                // than matched with the next prompt of its class.
                let cands: Vec<usize> = (0..queue.len()).filter(|&k| steps[queue[k]]["d"]["player"] == rd["player"] && steps[queue[k]]["d"]["cls"] == rd["cls"]).collect();
                let same = |k: usize| {
                    let od = norm_prompt(&steps[queue[k]]["d"]);
                    let rn = norm_prompt(&rd);
                    canon(&rn) == canon(&od) || canon(&card_set(&rn)) == canon(&card_set(&od))
                };
                let k = cands.iter().copied().find(|&k| same(k)).or(cands.first().copied());
                let Some(k) = k else {
                    dump_state(&g, -1);
                    return Outcome::Diverged { step: -1, what: "extra-prompt".into(), detail: format!("rust asks {}", canon(&norm_prompt(&rd))) };
                };
                let si = queue.remove(k);
                let od = &steps[si]["d"];
                let mut answer = steps[si]["a"].clone();
                if canon(&norm_prompt(&rd)) != canon(&norm_prompt(od)) && canon(&card_set(&norm_prompt(&rd))) == canon(&card_set(&norm_prompt(od))) {
                    // Same cards in another order (a discard pile's order is not observable):
                    // apply the oracle's answer to the same cards.
                    answer = remap_answer(&answer, &od["cards"], &rd["cards"]);
                } else if canon(&norm_prompt(&rd)) != canon(&norm_prompt(od)) {
                    dump_state(&g, si as isize);
                    return Outcome::Diverged {
                        step: si as isize,
                        what: "prompt".into(),
                        detail: format!("rust {}\noracle {}", canon(&norm_prompt(&rd)), canon(&norm_prompt(od))),
                    };
                }
                match g.decode_answer(&pr, &answer) {
                    Ok(res) => g.resolve(pi, res),
                    Err(e) => return Outcome::Diverged { step: si as isize, what: "decode".into(), detail: format!("{:?} for {}", e, canon(&answer)) },
                }
            }
            Pending::Turn(_) => {
                let Some(&ti) = turns.get(next_turn) else {
                    // The oracle game ended (or stopped) here; Rust wants another turn.
                    let res = &trace["result"];
                    if res["status"] != "finished" {
                        return finish(&g, decisions);
                    }
                    dump_state(&g, steps.len() as isize);
                    return Outcome::Diverged { step: steps.len() as isize, what: "obs".into(), detail: "rust continues after the oracle game ended".into() };
                };
                next_turn += 1;
                ptcg::expect::on_turn_decision(&g);
                if !scenario_done && g.st.turn >= ptcg::scenario::scenario_turn(scenario) {
                    scenario_done = true;
                    if let Err(e) = ptcg::scenario::apply(&mut g, scenario) {
                        return Outcome::Diverged { step: ti as isize, what: "scenario".into(), detail: e };
                    }
                    match ptcg::expect::parse(scenario) {
                        Ok(a) if !a.is_empty() => {
                            ptcg::expect::arm(&g, a, dump.is_some());
                            ptcg::expect::on_scenario_start(&g);
                        }
                        Ok(_) => {}
                        Err(e) => return Outcome::ExpectFailed { failures: vec![format!("invalid expect: {}", e)] },
                    }
                    let at = &trace["scenario"];
                    if at["step"].as_u64() != Some(ti as u64) || at["o"].as_str() != Some(g.observable_hash().as_str()) {
                        dump_state(&g, ti as isize);
                        return Outcome::Diverged { step: ti as isize, what: "scenario".into(), detail: format!("oracle applied at {}", at) };
                    }
                } else if g.observable_hash() != o_before(ti) {
                    dump_state(&g, ti as isize);
                    return Outcome::Diverged { step: ti as isize, what: "obs".into(), detail: format!("turn {} decision", g.st.turn) };
                }
                let d = &steps[ti]["d"];
                let opts = legal_turn_options(&g);
                let rust: Vec<Value> = opts.iter().map(|o| o.desc.clone()).collect();
                let oracle = d["options"].as_array().unwrap();
                if sorted_set(&rust) != sorted_set(oracle) {
                    let rs = sorted_set(&rust);
                    let os = sorted_set(oracle);
                    let only_r: Vec<&String> = rs.iter().filter(|x| !os.contains(x)).collect();
                    let only_o: Vec<&String> = os.iter().filter(|x| !rs.contains(x)).collect();
                    dump_state(&g, ti as isize);
                    return Outcome::Diverged { step: ti as isize, what: "options".into(), detail: format!("only rust {:?}\nonly oracle {:?}", only_r, only_o) };
                }
                if check_invariants() {
                    if let Err(e) = ptcg::invariants::check(&g) {
                        dump_state(&g, ti as isize);
                        return Outcome::Diverged { step: ti as isize, what: "invariant".into(), detail: e };
                    }
                }
                let want = canon(&steps[ti]["a"]);
                let k = opts.iter().position(|o| canon(&o.desc) == want).unwrap();
                let (q, pl) = segment(ti);
                queue = q;
                pool = pl;
                ptcg::rng::set_tape(Some(tape_of(&pool)));
                g.act(opts[k].action)
            }
            Pending::Finished => return finish(&g, decisions),
            other => return Outcome::Diverged { step: -1, what: "pending".into(), detail: format!("rust {:?}", other) },
        };
        if let Err(e) = r.and_then(|_| g.settle()) {
            // The oracle run stopped on the same GameError: parity holds if
            // the observable state left behind matches.
            let res = &trace["result"];
            if res["status"] == "error" && res["message"].as_str() == Some(e.0) {
                return finish(&g, decisions);
            }
            dump_state(&g, -1);
            return Outcome::Diverged { step: -1, what: "error".into(), detail: format!("{:?}", e) };
        }
    }
}

/// An approved divergence from `divergences.toml`: a trace
/// whose first divergence has kind `what` and a detail containing
/// `detail_contains` counts as approved, not diverged.
struct Approved {
    id: String,
    what: String,
    detail_contains: String,
}

/// Minimal reader for the `[[divergence]]` tables of `divergences.toml`
/// (string values only).
fn load_approved() -> Vec<Approved> {
    let path = std::env::var("PTCG_DIVERGENCES").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../divergences.toml").to_string());
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut out: Vec<Approved> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line == "[[divergence]]" {
            out.push(Approved { id: String::new(), what: String::new(), detail_contains: String::new() });
            continue;
        }
        let (Some(cur), Some((k, v))) = (out.last_mut(), line.split_once('=')) else { continue };
        let v = v.trim().trim_matches('"').to_string();
        match k.trim() {
            "id" => cur.id = v,
            "what" => cur.what = v,
            "detail_contains" => cur.detail_contains = v,
            _ => {}
        }
    }
    out.retain(|a| !a.what.is_empty() && !a.detail_contains.is_empty());
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let approved = load_approved();
    let mut files: Vec<PathBuf> = Vec::new();
    let mut dump: Option<PathBuf> = None;
    let mut quiet = false;
    // Observable-parity replay for every trace that records
    // observable hashes; `--strict` (or PTCG_OBS=0) forces lockstep state
    // equality, which older traces without them always get.
    let obs = !args.iter().any(|a| a == "--strict") && std::env::var("PTCG_OBS").map_or(true, |v| v != "0");
    if args.iter().any(|a| a == "--list-ported") {
        // Card keys ("Name SET NUM"), as in data/cards.json.
        for (i, d) in ptcg::carddb::cards().iter().enumerate() {
            if d.behavior.is_empty() || ptcg::cards::impl_for(i as u16).is_some() {
                println!("{}", d.full_name);
            }
        }
        return;
    }
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dump" => dump = it.next().map(PathBuf::from),
            "--quiet" => quiet = true,
            "--obs" | "--strict" | "--full" => {}
            "--threads" => {
                it.next();
            }
            _ => {
                let p = PathBuf::from(a);
                if p.is_dir() {
                    let mut v: Vec<PathBuf> =
                        std::fs::read_dir(&p).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|e| e == "json").unwrap_or(false) && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('g'))).collect();
                    v.sort();
                    files.extend(v);
                } else {
                    files.push(p);
                }
            }
        }
    }
    if let Some(d) = &dump {
        std::fs::create_dir_all(d).unwrap();
    }
    let (mut pass, mut fail, mut unsup, mut steps, mut appr) = (0, 0, 0, 0usize, 0usize);
    // Scenario `expect` accounting: games fully checked, games with a failed assertion, games where
    // an assertion's check point was never reached (the game ended first: not checked, not passed).
    let (mut exp_checked, mut exp_failed, mut exp_unchecked) = (0usize, 0usize, 0usize);
    let mut firsts: std::collections::BTreeMap<String, usize> = Default::default();
    // Replay on worker threads (the expect hooks, RNG tape and panic step are thread-local), then report
    // in file order.
    let threads = ptcg::selfplay::threads_from_args(&args);
    ptcg::selfplay::lower_priority(&args);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: std::sync::Mutex<Vec<Option<(Outcome, Option<ptcg::expect::Run>)>>> = std::sync::Mutex::new((0..files.len()).map(|_| None).collect());
    std::thread::scope(|s| {
        for _ in 0..threads.min(files.len()).max(1) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if i >= files.len() {
                    break;
                }
                let f = &files[i];
                let name = f.file_stem().unwrap().to_string_lossy().to_string();
                let trace: Value = match std::fs::read_to_string(f).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) {
                    Ok(v) => v,
                    Err(e) => {
                        results.lock().unwrap()[i] = Some((Outcome::Unsupported(format!("unreadable: {}", e)), None));
                        continue;
                    }
                };

                // A panic (e.g. a fixed-capacity list overflowing on a recorded state
                // with duplicated cards) fails this trace instead of the whole run.
                PANIC_STEP.with(|c| c.set(-1));
                ptcg::expect::take();
                ptcg::rng::set_tape(None);
                let out = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if obs && trace["start"]["o"].is_string() {
                        let out = replay_obs(&trace, dump.as_deref(), &name);
                        ptcg::rng::set_tape(None);
                        out
                    } else {
                        replay(&trace, dump.as_deref(), &name)
                    }
                })) {
                    Ok(o) => o,
                    Err(e) => {
                        let msg = e.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| e.downcast_ref::<String>().cloned()).unwrap_or_default();
                        Outcome::Diverged { step: PANIC_STEP.with(|c| c.get()), what: "panic".into(), detail: msg }
                    }
                };
                let run = ptcg::expect::take();
                results.lock().unwrap()[i] = Some((out, run));
            });
        }
    });
    let results = results.into_inner().unwrap();
    for (f, r) in files.iter().zip(results) {
        let name = f.file_stem().unwrap().to_string_lossy().to_string();
        let (mut out, run) = r.unwrap();
        if let (Outcome::Pass { .. }, Some(run)) = (&out, &run) {
            let mut failures: Vec<String> = Vec::new();
            for fl in &run.failures {
                let at = match fl.at {
                    ptcg::expect::At::TurnEnd => "turn_end",
                    ptcg::expect::At::NextTurn => "next_turn",
                    ptcg::expect::At::NextTurnEnd => "next_turn_end",
                    ptcg::expect::At::GameEnd => "game_end",
                    ptcg::expect::At::Tiebreaker => "tiebreaker",
                    ptcg::expect::At::Decision => "decision",
                };
                failures.push(format!("assertion #{} ({}) {}\n  cite: {}\n  actual: {}", fl.index, at, fl.assertion, fl.cite, fl.actual));
                if let (Some(dir), Some(state)) = (&dump, &fl.state) {
                    let _ = std::fs::write(dir.join(format!("{}.expect{}.rust.json", name, fl.index)), state);
                }
            }
            if run.checked() == run.assertions.len() {
                exp_checked += 1;
            } else {
                exp_unchecked += 1;
                if !quiet {
                    println!("EXPECT NOT CHECKED {}: {} of {} assertions reached their check point", name, run.checked(), run.assertions.len());
                }
            }
            if !failures.is_empty() {
                exp_failed += 1;
                out = Outcome::ExpectFailed { failures };
            }
        } else if let Outcome::ExpectFailed { .. } = &out {
            exp_failed += 1;
        }
        match &out {
            Outcome::ExpectFailed { failures, .. } => {
                for l in failures {
                    println!("EXPECT FAILED {}: {}", f.display(), l);
                }
            }
            Outcome::Pass { steps: s } => {
                pass += 1;
                steps += s;
                if !quiet {
                    println!("PASS {} ({} steps)", name, s);
                }
            }
            Outcome::Diverged { step, what, detail } if approved.iter().any(|a| a.what == *what && detail.contains(&a.detail_contains)) => {
                appr += 1;
                let id = approved.iter().find(|a| a.what == *what && detail.contains(&a.detail_contains)).map(|a| a.id.as_str()).unwrap_or("");
                if !quiet {
                    println!("APPROVED {} at step {}: {} ({})", name, step, what, id);
                }
            }
            Outcome::Diverged { step, what, detail } => {
                fail += 1;
                *firsts.entry(what.clone()).or_default() += 1;
                println!("DIVERGED {} at step {}: {}\n  {}", f.display(), step, what, detail.replace('\n', "\n  "));
            }
            Outcome::Unsupported(why) => {
                unsup += 1;
                if !quiet {
                    println!("SKIP {}: {}", name, why);
                }
            }
        }
    }
    println!("\n{} traces: {} pass ({} steps), {} diverged, {} unsupported, {} approved", files.len(), pass, steps, fail, unsup, appr);
    if exp_checked + exp_failed + exp_unchecked > 0 {
        println!("expect: {} games checked, {} failed, {} not checked", exp_checked, exp_failed, exp_unchecked);
    }
    for (k, v) in firsts {
        println!("  first divergence {}: {}", k, v);
    }
    if fail > 0 || exp_failed > 0 {
        std::process::exit(1);
    }
}
