//! Self-play in the Rust engine alone: one game from two decks and a seed,
//! with a driver policy, an optional scenario (board edits at the first turn
//! decision on or after `scenario.turn`, scripted `answers`, `expect`
//! assertions) and an optional trace in the oracle's trace format, which
//! `diff` replays. Used by `scen` (scenarios) and `fuzz` (fresh-seed
//! self-play, the oracle-free tier 4).
//!
//! Every game checks the PLAN.md 4.6 invariants at each turn decision and
//! fails on engine errors, stuck prompts and turns without legal options.

use crate::carddb::{def, def_by_full_name, DefId};
use crate::game::{Action, Game, Pending};
use crate::list::CardList;
use crate::options::{legal_turn_options, TurnOption};
use crate::rng::{Draw, Rng};
use serde_json::{json, Value};
use std::collections::VecDeque;

thread_local! {
    /// The decision being resolved (descriptor or answer), reported when the game panics.
    static LAST: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn note(what: impl FnOnce() -> String) {
    LAST.with(|l| *l.borrow_mut() = what());
}

/// The oracle runner's caps.
pub const MAX_STEPS: usize = 4000;
pub const MAX_TURNS: i32 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Anything but pass, pass 8% of the time.
    Random,
    /// Develop the board first (play cards, Abilities, Stadium), then attack.
    Heur,
}

impl Policy {
    /// The oracle's policy names; the oracle's `bot` and `mix:*` have no Rust bot and play `heur`.
    pub fn parse(s: &str) -> Policy {
        if s == "random" {
            Policy::Random
        } else {
            Policy::Heur
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Policy::Random => "random",
            Policy::Heur => "heur",
        }
    }
}

pub struct Opts<'a> {
    pub seed: u32,
    pub decks: [&'a [DefId]; 2],
    pub policy: [Policy; 2],
    /// A scenario already passed through [`load_scenario`].
    pub scenario: Option<&'a Value>,
    pub record: bool,
}

pub enum End {
    /// Finished (`finished`) or stopped at the step or turn cap (`cap`).
    Done { status: &'static str },
    /// An engine failure: error, stuck prompt, invariant, panic.
    Fail(String),
    /// The scenario itself is broken: its edits failed, or a scripted answer was not offered.
    Broken(String),
}

pub struct Played {
    pub end: End,
    /// Whether the scenario's edits were applied (always false without a scenario).
    pub reached: bool,
    /// The scenario's `expect` run, when it had assertions.
    pub expect: Option<crate::expect::Run>,
    pub trace: Option<Value>,
    pub turn: i32,
    pub winner: i8,
}

/// Deck lines (`"4 Name"` or `"Name"`; English keys, official or Twinleaf names) to card ids.
pub fn expand_deck(lines: &Value) -> Result<Vec<DefId>, String> {
    let mut out = Vec::new();
    for l in lines.as_array().ok_or("deck: not a list")? {
        let l = l.as_str().ok_or("deck: not a string")?;
        let (n, name) = match l.split_once(' ') {
            Some((n, rest)) if n.parse::<usize>().is_ok() => (n.parse::<usize>().unwrap(), rest),
            _ => (1, l),
        };
        let d = def_by_full_name(name).ok_or_else(|| format!("unknown card {}", name))?;
        out.extend(std::iter::repeat(d).take(n));
    }
    Ok(out)
}

fn tl(name: &str) -> Result<String, String> {
    def_by_full_name(name).map(|d| def(d).full_name.to_string()).ok_or_else(|| format!("unknown card {}", name))
}

/// The scenario with every card name mapped to its official full name (international key) and move
/// names in scripted answers to official names, as the turn option descriptors use them. Old Twinleaf
/// names are still accepted as input.
pub fn load_scenario(sc: &Value) -> Result<Value, String> {
    let mut sc = sc.clone();
    let mut seen: Vec<DefId> = Vec::new();
    fn one(v: &mut Value, seen: &mut Vec<DefId>) -> Result<(), String> {
        if let Some(s) = v.as_str() {
            let d = def_by_full_name(s).ok_or_else(|| format!("unknown card {}", s))?;
            seen.push(d);
            *v = Value::from(def(d).full_name);
        }
        Ok(())
    }
    fn many(v: &mut Value, seen: &mut Vec<DefId>) -> Result<(), String> {
        let ds = expand_deck(v)?;
        seen.extend(ds.iter().copied());
        *v = Value::Array(ds.iter().map(|d| Value::from(def(*d).full_name)).collect());
        Ok(())
    }
    fn stack(v: &mut Value, seen: &mut Vec<DefId>) -> Result<(), String> {
        if v.is_string() {
            one(v, seen)
        } else {
            many(v, seen)
        }
    }
    for side in ["me", "opp"] {
        let Some(d) = sc.get_mut(side).and_then(|d| d.as_object_mut()) else { continue };
        for k in ["discard", "hand", "deck_top", "prizes", "active_energy"] {
            if let Some(v) = d.get_mut(k) {
                many(v, &mut seen)?;
            }
        }
        for k in ["stadium", "active_tool"] {
            if let Some(v) = d.get_mut(k) {
                one(v, &mut seen)?;
            }
        }
        if let Some(v) = d.get_mut("active") {
            stack(v, &mut seen)?;
        }
        if let Some(bench) = d.get_mut("bench").and_then(|b| b.as_array_mut()) {
            for b in bench {
                if let Some(v) = b.get_mut("card") {
                    stack(v, &mut seen)?;
                }
                if let Some(v) = b.get_mut("energy") {
                    many(v, &mut seen)?;
                }
                if let Some(v) = b.get_mut("tool") {
                    one(v, &mut seen)?;
                }
            }
        }
    }
    if let Some(decks) = sc.get("decks").and_then(|d| d.as_array()) {
        for d in decks {
            seen.extend(expand_deck(d)?);
        }
    }
    // Move names: official or Twinleaf, among the cards the scenario mentions.
    let move_name = |name: &str| -> String {
        for d in &seen {
            let c = def(*d);
            if let Some(a) = c.attacks.iter().find(|a| a.name == name || a.tl_name == name) {
                return a.name.to_string();
            }
            if let Some(p) = c.powers.iter().find(|p| p.name == name || p.tl_name == name) {
                return p.name.to_string();
            }
        }
        name.to_string()
    };
    let mut answers = sc.get("answers").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    for a in answers.iter_mut() {
        let Some(o) = a.as_object_mut() else { continue };
        let kind = o.get("a").and_then(|x| x.as_str()).unwrap_or("").to_string();
        if kind == "play" {
            if let Some(c) = o.get("card").and_then(|c| c.as_str()).filter(|c| !c.contains('#')).map(|c| c.to_string()) {
                o.insert("card".into(), Value::from(tl(&c)?));
            }
        }
        if ["attack", "ability", "trainerAbility"].contains(&kind.as_str()) {
            if let Some(n) = o.get("name").and_then(|x| x.as_str()).map(|s| s.to_string()) {
                o.insert("name".into(), Value::from(move_name(&n)));
            }
        }
        if kind == "attack" {
            if let Some(f) = o.get("from").and_then(|x| x.as_str()).map(|s| s.to_string()) {
                o.insert("from".into(), Value::from(tl(&f)?));
            }
        }
        if let Some(n) = o.get("attack").and_then(|x| x.as_str()).map(|s| s.to_string()) {
            o.insert("attack".into(), Value::from(move_name(&n)));
        }
    }
    if sc.get("answers").is_some() {
        sc["answers"] = Value::Array(answers);
    }
    Ok(sc)
}

/// Canonical JSON with sorted keys (the oracle's `stableStringify`).
pub fn stable(v: &Value) -> String {
    match v {
        Value::Object(o) => {
            let mut ks: Vec<&String> = o.keys().collect();
            ks.sort();
            let parts: Vec<String> = ks.iter().map(|k| format!("{}:{}", serde_json::to_string(k).unwrap(), stable(&o[*k]))).collect();
            format!("{{{}}}", parts.join(","))
        }
        Value::Array(a) => format!("[{}]", a.iter().map(stable).collect::<Vec<_>>().join(",")),
        _ => v.to_string(),
    }
}

fn is_turn_answer(v: &Value) -> bool {
    v.get("a").is_some_and(|a| a.is_string())
}

/// The oracle runner's `findScriptedByCard`: a scripted play that names its card instead of its
/// instance id (ids depend on the shuffle).
fn find_by_card(g: &Game, opts: &[TurnOption], scripted: &Value) -> Option<usize> {
    if let Some(prefix) = scripted.get("card_prefix").and_then(|x| x.as_str()) {
        return opts.iter().position(|o| o.desc["a"] == scripted["a"] && o.desc["card"].as_str().is_some_and(|c| c.starts_with(prefix)));
    }
    if scripted["a"] != "play" {
        return None;
    }
    let name = scripted.get("name").and_then(|x| x.as_str()).or_else(|| scripted.get("card").and_then(|x| x.as_str()))?;
    if name.contains('#') {
        return None;
    }
    let want_def = def_by_full_name(name);
    let want_target = scripted.get("target").map(stable);
    let p = g.st.active_player as usize;
    opts.iter().position(|o| {
        let Action::PlayCard { hand_index, .. } = o.action else { return false };
        let Some(c) = g.st.players[p].hand.get(hand_index as usize) else { return false };
        let d = g.st.cdef(c);
        let named = want_def == Some(g.st.cards[c as usize].def) || crate::carddb::card_is(d, name) || d.name == name || d.tl_name == name;
        named && want_target.as_ref().map_or(true, |t| stable(&o.desc["target"]) == *t)
    })
}

/// A turn option by policy, as the oracle runner's `decideTurn`.
pub fn pick_turn(opts: &[TurnOption], policy: Policy, rng: &mut Rng) -> usize {
    let kind = |o: &TurnOption| o.desc["a"].as_str().unwrap_or("").to_string();
    let pass = opts.iter().position(|o| kind(o) == "pass");
    match policy {
        Policy::Heur => {
            let idx = |ks: &[&str]| -> Vec<usize> { (0..opts.len()).filter(|&i| ks.contains(&kind(&opts[i]).as_str())).collect() };
            let develop = idx(&["play", "ability", "trainerAbility", "energyAbility", "stadium"]);
            let attack = idx(&["attack"]);
            let retreat = idx(&["retreat"]);
            let r = rng.below(1_000_000) as f64 / 1e6;
            if !develop.is_empty() && r < 0.8 {
                return develop[rng.index(develop.len())];
            }
            if !attack.is_empty() && r < 0.97 {
                return attack[rng.index(attack.len())];
            }
            if !retreat.is_empty() && r < 0.99 {
                return retreat[rng.index(retreat.len())];
            }
            pass.unwrap_or_else(|| rng.index(opts.len()))
        }
        Policy::Random => {
            let others: Vec<usize> = (0..opts.len()).filter(|i| Some(*i) != pass).collect();
            if others.is_empty() || (pass.is_some() && rng.below(1_000_000) < 80_000) {
                return pass.unwrap_or(0);
            }
            others[rng.index(others.len())]
        }
    }
}

/// A random valid prompt answer from the selection masks; when nothing can be chosen, cancel or an
/// empty answer if the prompt takes one (the oracle's `randomAnswer` fallbacks).
fn random_prompt(g: &mut Game, pi: usize, rng: &mut Rng) -> Option<Value> {
    let picked = (|| {
        let sel = g.select().ok()??;
        let n = sel.options.len();
        let mut picks: Vec<usize> = Vec::new();
        loop {
            let (mask, stop) = g.pick_mask(&sel, &picks);
            let allowed: Vec<usize> = (0..n).filter(|j| mask[*j]).collect();
            let choices = allowed.len() + stop as usize;
            if choices == 0 {
                if picks.is_empty() && !stop {
                    return None;
                }
                break;
            }
            let r = rng.index(choices);
            if r == allowed.len() {
                break;
            }
            picks.push(allowed[r]);
            if sel.max_count <= 1 {
                break;
            }
        }
        sel.wire_answer(&picks).ok()
    })();
    picked.or_else(|| {
        let pr = g.prompts.as_slice()[pi];
        [Value::Null, Value::Array(vec![])].into_iter().find(|w| g.decode_answer(&pr, w).is_ok())
    })
}

fn chance_json(d: &[Draw]) -> Value {
    Value::Array(
        d.iter()
            .map(|d| match d {
                Draw::Coin(v) => json!({ "k": "coin", "v": v }),
                Draw::Shuffle(v) => json!({ "k": "shuffle", "v": v }),
                Draw::Index(n, v) => json!({ "k": "index", "n": n, "v": v }),
            })
            .collect(),
    )
}

/// The trace being written, in the oracle's format (`runner.ts` `Trace`).
struct Rec {
    on: bool,
    steps: Vec<Value>,
    start: Value,
    scenario_at: Value,
}

impl Rec {
    fn step(&mut self, g: &Game, p: usize, d: Value, a: Value) {
        if !self.on {
            return;
        }
        let c = chance_json(&crate::rng::take_recorded());
        self.steps.push(json!({ "i": self.steps.len(), "p": p, "d": d, "a": a, "c": c, "h": g.state_hash(), "o": g.observable_hash() }));
    }
}

/// Play one game. Runs on the calling thread (the `expect` hooks and the chance recording are
/// thread-local).
pub fn play(o: &Opts) -> Played {
    crate::expect::take();
    crate::rng::take_recorded();
    let mut rec = Rec { on: o.record, steps: Vec::new(), start: Value::Null, scenario_at: Value::Null };
    let mut g = Game::new(o.seed);
    if o.record {
        g.rng = g.rng.record();
    }
    let mut reached = false;
    let end = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(o, &mut g, &mut rec, &mut reached)))
        .unwrap_or_else(|e| {
            let msg = e.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| e.downcast_ref::<String>().cloned()).unwrap_or_default();
            End::Fail(format!("panic: {} (turn {}, resolving {})", msg, g.st.turn, LAST.with(|l| l.borrow().clone())))
        });
    let expect = crate::expect::take();
    crate::rng::take_recorded();
    let trace = o.record.then(|| {
        let (status, message) = match &end {
            End::Done { status } => (*status, None),
            End::Fail(m) => ("error", Some(m.clone())),
            End::Broken(m) => ("error", Some(m.clone())),
        };
        let deck_names = |p: usize| -> Vec<&str> { o.decks[p].iter().map(|d| def(*d).full_name).collect() };
        let mut t = json!({
            "header": {
                "seed": o.seed,
                "decks": [deck_names(0), deck_names(1)],
                "policy": [o.policy[0].name(), o.policy[1].name()],
                "engine": "rust",
                "scenario": o.scenario.cloned().unwrap_or(Value::Null),
            },
            "start": rec.start,
            "steps": rec.steps,
            "result": { "status": status, "winner": g.st.winner, "turn": g.st.turn, "message": message },
        });
        if !rec.scenario_at.is_null() {
            t["scenario"] = rec.scenario_at;
        }
        t
    });
    Played { end, reached, expect, trace, turn: g.st.turn, winner: g.st.winner }
}

fn run(o: &Opts, g: &mut Game, rec: &mut Rec, reached: &mut bool) -> End {
    let started = g.start(o.decks).and_then(|_| g.settle());
    if rec.on {
        let c = chance_json(&crate::rng::take_recorded());
        rec.start = json!({ "c": c, "h": g.state_hash(), "o": g.observable_hash() });
    }
    if let Err(e) = started {
        return End::Fail(format!("start: {:?}", e));
    }
    let mut prng = Rng::new(o.seed.wrapping_mul(2654435761));
    let mut script: VecDeque<Value> = VecDeque::new();
    let turn_at = o.scenario.map(crate::scenario::scenario_turn);
    for step in 0..MAX_STEPS {
        if g.st.turn > MAX_TURNS {
            break;
        }
        let r = match g.pending() {
            Pending::Finished => return End::Done { status: "finished" },
            Pending::Stuck => return End::Fail(format!("stuck at step {} (turn {})", step, g.st.turn)),
            Pending::Chance(_) | Pending::Info(_) => g.settle(),
            Pending::Decision(pi) => {
                let pr = g.prompts.as_slice()[pi];
                let p = g.st.player_index_by_id(pr.player_id);
                let wire = match script.front().filter(|v| !is_turn_answer(v)).cloned() {
                    Some(v) => {
                        script.pop_front();
                        v
                    }
                    None => match random_prompt(g, pi, &mut prng) {
                        Some(w) => w,
                        None => return End::Fail(format!("no valid answer at step {} (turn {}): {}", step, g.st.turn, g.describe_prompt(&pr))),
                    },
                };
                let d = if rec.on { g.describe_prompt(&pr) } else { Value::Null };
                note(|| format!("prompt {} with {}", g.describe_prompt(&pr)["cls"], wire));
                let r = match g.decode_answer(&pr, &wire) {
                    Ok(res) => g.resolve(pi, res),
                    Err(e) => return End::Broken(format!("answer {} rejected: {:?} by {}", wire, e, g.describe_prompt(&pr))),
                };
                let r = r.and_then(|_| g.settle());
                if r.is_ok() {
                    rec.step(g, p, d, wire);
                }
                r
            }
            Pending::Turn(_) => {
                crate::expect::on_turn_decision(g);
                if let (Some(sc), Some(at)) = (o.scenario, turn_at) {
                    if !*reached && g.st.turn >= at {
                        *reached = true;
                        if let Err(e) = crate::scenario::apply(g, sc) {
                            return End::Broken(e);
                        }
                        match crate::expect::parse(sc) {
                            Ok(a) if !a.is_empty() => {
                                crate::expect::arm(g, a, false);
                                crate::expect::on_scenario_start(g);
                            }
                            Ok(_) => {}
                            Err(e) => return End::Broken(format!("invalid expect: {}", e)),
                        }
                        script = sc["answers"].as_array().cloned().unwrap_or_default().into();
                        if rec.on {
                            crate::rng::take_recorded();
                            rec.scenario_at = json!({ "step": rec.steps.len(), "h": g.state_hash(), "o": g.observable_hash() });
                        }
                    }
                }
                let opts = legal_turn_options(g);
                if opts.is_empty() {
                    return End::Fail(format!("no legal turn options at step {} (turn {})", step, g.st.turn));
                }
                // Scripted prompt answers left over: the oracle asked a prompt Rust doesn't (a "you may"
                // with no possible effect, PLAN.md 8.5), so they have nothing to answer.
                while script.front().is_some_and(|v| !is_turn_answer(v)) {
                    script.pop_front();
                }
                let p = g.st.active_player as usize;
                let k = match script.pop_front() {
                    Some(want) => {
                        let key = stable(&want);
                        match opts.iter().position(|x| stable(&x.desc) == key).or_else(|| find_by_card(g, &opts, &want)) {
                            Some(k) => k,
                            None => return End::Broken(format!("scripted answer not among options: {}", key)),
                        }
                    }
                    None => pick_turn(&opts, o.policy[p], &mut prng),
                };
                let d = if rec.on { json!({ "kind": "turn", "options": opts.iter().map(|x| x.desc.clone()).collect::<Vec<_>>() }) } else { Value::Null };
                note(|| format!("turn action {}", opts[k].desc));
                let r = g.act(opts[k].action).and_then(|_| g.settle());
                if r.is_ok() {
                    rec.step(g, p, d, opts[k].desc.clone());
                }
                r
            }
        };
        if let Err(e) = r {
            return End::Fail(format!("error at step {} (turn {}): {:?}", step, g.st.turn, e));
        }
        if matches!(g.pending(), Pending::Turn(_)) {
            if let Err(e) = crate::invariants::check(g) {
                return End::Fail(format!("invariant at step {} (turn {}): {}", step, g.st.turn, e));
            }
        }
    }
    End::Done { status: "cap" }
}

extern "C" {
    fn setpriority(which: i32, who: u32, prio: i32) -> i32;
}

/// Worker threads for a self-play run: `--threads T` if given; else every core with `--full` or
/// off macOS (the box), and half the cores on a Mac so the laptop stays usable.
pub fn threads_from_args(args: &[String]) -> usize {
    if let Some(t) = args.iter().position(|a| a == "--threads").and_then(|i| args.get(i + 1)) {
        return t.parse().expect("--threads");
    }
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    if cfg!(target_os = "macos") && !args.iter().any(|a| a == "--full") {
        (cores / 2).max(1)
    } else {
        cores
    }
}

/// Run at low priority (nice 10) unless `--full`, so a long run yields to interactive work.
/// Call before spawning the worker threads, which inherit it.
pub fn lower_priority(args: &[String]) {
    if !args.iter().any(|a| a == "--full") {
        // PRIO_PROCESS, this process.
        unsafe {
            setpriority(0, 0, 10);
        }
    }
}
