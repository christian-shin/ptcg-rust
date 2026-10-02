//! Replay oracle traces through the Rust engine and stop at the first
//! divergence (PLAN.md 4.3).
//!
//!   diff <trace.json|dir>... [--dump <dir>] [--quiet]
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

#[derive(Debug)]
enum Outcome {
    Pass { steps: usize },
    Diverged { step: isize, what: String, detail: String },
    Unsupported(String),
}

fn canon(v: &Value) -> String {
    serde_json::to_string(v).unwrap()
}

fn sorted_set(v: &[Value]) -> Vec<String> {
    let mut s: Vec<String> = v.iter().map(canon).collect();
    s.sort();
    s
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
            return Outcome::Unsupported(format!("card not ported: {}", ptcg::carddb::def(*d).full_name));
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
                if !scenario_done && g.st.turn >= ptcg::scenario::scenario_turn(scenario) {
                    scenario_done = true;
                    if let Err(e) = ptcg::scenario::apply(&mut g, scenario) {
                        return Outcome::Diverged { step: i as isize, what: "scenario".into(), detail: e };
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
    }
    Outcome::Pass { steps: steps.len() }
}

/// An approved divergence from `divergences.toml` (PLAN.md 4.7): a trace
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
    if args.iter().any(|a| a == "--list-ported") {
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
            _ => {
                let p = PathBuf::from(a);
                if p.is_dir() {
                    let mut v: Vec<PathBuf> =
                        std::fs::read_dir(&p).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|e| e == "json").unwrap_or(false)).collect();
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
    let mut firsts: std::collections::BTreeMap<String, usize> = Default::default();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        let trace: Value = serde_json::from_str(&text).unwrap();
        let name = f.file_stem().unwrap().to_string_lossy().to_string();
        // A panic (e.g. a fixed-capacity list overflowing on a Twinleaf state
        // with duplicated cards) fails this trace instead of the whole run.
        PANIC_STEP.with(|c| c.set(-1));
        let out = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| replay(&trace, dump.as_deref(), &name))) {
            Ok(o) => o,
            Err(e) => {
                let msg = e.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| e.downcast_ref::<String>().cloned()).unwrap_or_default();
                Outcome::Diverged { step: PANIC_STEP.with(|c| c.get()), what: "panic".into(), detail: msg }
            }
        };
        match &out {
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
                println!("DIVERGED {} at step {}: {}\n  {}", name, step, what, detail.replace('\n', "\n  "));
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
    for (k, v) in firsts {
        println!("  first divergence {}: {}", k, v);
    }
    if fail > 0 {
        std::process::exit(1);
    }
}
