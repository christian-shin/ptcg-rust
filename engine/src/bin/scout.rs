//! Pick oracle games worth running: play many candidate games in Rust
//! (fast), score each by how often the target cards act, and write the best,
//! most varied ones as (seed, decks, answers) for the oracle's `replay`
//! command. Every slow oracle game then exercises the cards under test.
//!
//!   scout <spec.json> <out.json> --targets "Full Name A|Full Name B"
//!         [--candidates N] [--keep K] [--seed S] [--max-steps M]
//!
//! `spec.json` is the `check_cards.py` corpus spec (`{"decks": [{name, cards}]}`).
//! Answers are recorded in the oracle wire format, exactly what a trace's
//! `a` field holds, so `diff` can compare the oracle replay step by step.
//! A Rust port that is wrong can make a scouted answer illegal in the
//! oracle; the replay then stops there and `diff` reports the divergence.

use ptcg::carddb::def_by_full_name;
use ptcg::list::CardList;
use ptcg::game::{Action, Game, Pending};
use ptcg::options::legal_turn_options;
use ptcg::rng::Rng;
use ptcg::types::SlotType;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

struct Candidate {
    seed: u32,
    decks: [usize; 2],
    answers: Vec<Value>,
    score: f64,
    sig: BTreeMap<String, u32>,
}

fn arg<'a>(args: &'a [String], k: &str) -> Option<&'a str> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

/// The card an action is "about", if any: played card, attacker, ability source, stadium.
fn action_card(g: &Game, p: usize, a: &Action) -> Option<u8> {
    let pl = &g.st.players[p];
    match *a {
        Action::PlayCard { hand_index, .. } => pl.hand.get(hand_index as usize),
        Action::Attack { .. } => g.st.active_pokemon(p),
        Action::UseAbility { target, .. } | Action::UseTrainerAbility { target, .. } => match target.slot {
            SlotType::Hand => pl.hand.get(target.index as usize),
            SlotType::Discard => pl.discard.get(target.index as usize),
            _ => ptcg::prompts::get_target(&g.st, p, target).ok().and_then(|t| g.st.slot_pokemon(t.p as usize, t.s)),
        },
        Action::UseStadium => g.st.stadium_card(),
        _ => None,
    }
}

/// Distinct effect types produced since the last clear, in first-seen order:
/// different branches of a card's logic produce different sequences.
fn take_effects() -> String {
    ptcg::game::EFFECT_TRACE.with(|v| {
        let mut v = v.borrow_mut();
        let mut seen: Vec<&'static str> = Vec::new();
        for t in v.iter() {
            if !seen.contains(t) {
                seen.push(t);
            }
        }
        v.clear();
        seen.iter().map(|t| t.trim_end_matches("_EFFECT")).collect::<Vec<_>>().join(",")
    })
}

fn clear_effects() {
    ptcg::game::EFFECT_TRACE.with(|v| v.borrow_mut().clear());
}

fn kind(a: &Action) -> &'static str {
    match a {
        Action::PlayCard { .. } => "play",
        Action::Attack { .. } => "attack",
        Action::UseAbility { .. } | Action::UseTrainerAbility { .. } => "ability",
        Action::UseStadium => "stadium",
        Action::Retreat { .. } => "retreat",
        Action::Pass => "pass",
    }
}

/// `heur` policy, as in the oracle runner: develop first, then attack.
fn pick_turn(opts: &[ptcg::options::TurnOption], rng: &mut Rng) -> usize {
    let idx = |ks: &[&str]| -> Vec<usize> { (0..opts.len()).filter(|&i| ks.contains(&kind(&opts[i].action))).collect() };
    let develop = idx(&["play", "ability", "stadium"]);
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
    opts.iter().position(|o| matches!(o.action, Action::Pass)).unwrap_or_else(|| rng.index(opts.len()))
}

fn play(decks: [&[u16]; 2], seed: u32, targets: &HashSet<u16>, max_steps: usize, rng: &mut Rng) -> Option<(Vec<Value>, f64, BTreeMap<String, u32>)> {
    let mut g = Game::new(seed);
    g.trace_effects = true;
    g.start(decks).ok()?;
    g.settle().ok()?;
    let mut answers = Vec::new();
    let mut score = 0.0;
    let mut sig: BTreeMap<String, u32> = BTreeMap::new();
    // The target action whose follow-up prompts are being answered this turn.
    let mut context: Option<String> = None;
    for _ in 0..max_steps {
        if g.st.phase == ptcg::types::GamePhase::Finished {
            break;
        }
        match g.pending() {
            Pending::Turn(_) => {
                let opts = legal_turn_options(&g);
                if opts.is_empty() {
                    break;
                }
                let k = pick_turn(&opts, rng);
                let p = g.st.active_player as usize;
                context = None;
                if let Some(c) = action_card(&g, p, &opts[k].action) {
                    let d = g.st.cards[c as usize].def;
                    if targets.contains(&d) {
                        score += 1.0;
                        context = Some(format!("{}:{}", kind(&opts[k].action), ptcg::carddb::def(d).full_name));
                    }
                }
                answers.push(opts[k].desc.clone());
                clear_effects();
                let r = g.act(opts[k].action).and_then(|_| g.settle());
                let fx = take_effects();
                if let Some(ctx) = &context {
                    *sig.entry(format!("{} [{}]", ctx, fx)).or_default() += 1;
                }
                if r.is_err() {
                    break;
                }
            }
            Pending::Decision(pi) => {
                let sel = match g.select() {
                    Ok(Some(s)) => s,
                    _ => break,
                };
                // Random valid picks from the same masks VecEnv uses.
                let n = sel.options.len();
                let mut picks: Vec<usize> = Vec::new();
                loop {
                    let (mask, stop) = g.pick_mask(&sel, &picks);
                    let allowed: Vec<usize> = (0..n).filter(|j| mask[*j]).collect();
                    let choices = allowed.len() + stop as usize;
                    if choices == 0 {
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
                let wire = match sel.wire_answer(&picks) {
                    Ok(w) => w,
                    Err(_) => break,
                };
                let pr = g.prompts.as_slice()[pi];
                // Prompts raised by a target card count as activity too.
                let res = match g.decode_answer(&pr, &wire) {
                    Ok(r) => r,
                    Err(_) => break, // no valid answer (stuck): end the candidate here
                };
                answers.push(wire);
                clear_effects();
                let r = g.resolve(pi, res).and_then(|_| g.settle());
                let fx = take_effects();
                if let Some(ctx) = &context {
                    *sig.entry(format!("{} > {} [{}]", ctx, picks.len(), fx)).or_default() += 1;
                }
                if r.is_err() {
                    break;
                }
            }
            _ => break,
        }
    }
    // Reward reaching later turns with the targets around (states cards rarely see early).
    score += (g.st.turn as f64).min(40.0) * 0.01;
    Some((answers, score, sig))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: scout <spec.json> <out.json> --targets \"A|B\" [--candidates N] [--keep K] [--seed S] [--max-steps M]");
        std::process::exit(2);
    }
    let spec: Value = serde_json::from_str(&std::fs::read_to_string(&args[0]).expect("spec")).expect("spec json");
    let out = &args[1];
    let targets_arg = arg(&args, "--targets").unwrap_or("");
    let candidates: usize = arg(&args, "--candidates").map(|s| s.parse().unwrap()).unwrap_or(2000);
    let keep: usize = arg(&args, "--keep").map(|s| s.parse().unwrap()).unwrap_or(32);
    let seed0: u32 = arg(&args, "--seed").map(|s| s.parse().unwrap()).unwrap_or(1);
    let max_steps: usize = arg(&args, "--max-steps").map(|s| s.parse().unwrap()).unwrap_or(400);
    let mut targets = HashSet::new();
    for t in targets_arg.split('|').filter(|s| !s.is_empty()) {
        match def_by_full_name(t) {
            Some(d) => {
                targets.insert(d);
            }
            None => {
                eprintln!("unknown target card: {}", t);
                std::process::exit(2);
            }
        }
    }
    let deck_specs = spec["decks"].as_array().expect("spec.decks");
    let mut names: Vec<String> = Vec::new();
    let mut decks: Vec<Vec<u16>> = Vec::new();
    for d in deck_specs {
        names.push(d["name"].as_str().unwrap_or("").to_string());
        let mut v = Vec::new();
        for c in d["cards"].as_array().unwrap() {
            let n = c.as_str().unwrap();
            match def_by_full_name(n) {
                Some(id) => v.push(id),
                None => {
                    eprintln!("unknown card {}", n);
                    std::process::exit(2);
                }
            }
        }
        decks.push(v);
    }
    let mut rng = Rng::new(seed0 ^ 0x5c07);
    let mut pool: Vec<Candidate> = Vec::new();
    let t0 = std::time::Instant::now();
    for i in 0..candidates {
        let seed = seed0.wrapping_mul(1_000_003).wrapping_add(i as u32);
        let a = rng.index(decks.len());
        let b = rng.index(decks.len());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut prng = Rng::new(seed ^ 0x9e37_79b9);
            play([&decks[a], &decks[b]], seed, &targets, max_steps, &mut prng)
        }));
        if let Ok(Some((answers, score, sig))) = r {
            pool.push(Candidate { seed, decks: [a, b], answers, score, sig });
        }
    }
    let scouted = pool.len();
    // Greedy selection: best score, discounted by how often its activity
    // signature (which target did what) is already covered.
    let mut covered: BTreeMap<String, u32> = BTreeMap::new();
    let mut chosen: Vec<Candidate> = Vec::new();
    while chosen.len() < keep && !pool.is_empty() {
        let value = |c: &Candidate, covered: &BTreeMap<String, u32>| -> f64 {
            let novelty: f64 = c.sig.iter().map(|(k, v)| (*v as f64).min(4.0) / (1.0 + *covered.get(k).unwrap_or(&0) as f64)).sum();
            novelty + c.score * 0.05
        };
        let (bi, _) = pool.iter().enumerate().map(|(i, c)| (i, value(c, &covered))).fold((0, f64::MIN), |acc, x| if x.1 > acc.1 { x } else { acc });
        let c = pool.swap_remove(bi);
        for (k, v) in &c.sig {
            *covered.entry(k.clone()).or_default() += (*v).min(4);
        }
        chosen.push(c);
    }
    let games: Vec<Value> = chosen
        .iter()
        .map(|c| {
            let deck_names = |i: usize| -> Vec<&str> { decks[i].iter().map(|d| ptcg::carddb::def(*d).full_name).collect() };
            json!({
                "seed": c.seed,
                "decks": [deck_names(c.decks[0]), deck_names(c.decks[1])],
                "deckNames": [names[c.decks[0]], names[c.decks[1]]],
                "answers": c.answers,
                "score": c.score,
                "sig": c.sig,
            })
        })
        .collect();
    std::fs::write(out, serde_json::to_string(&games).unwrap()).expect("write out");
    let mut total: BTreeMap<String, u32> = BTreeMap::new();
    for c in &chosen {
        for (k, v) in &c.sig {
            *total.entry(k.clone()).or_default() += v;
        }
    }
    eprintln!("scouted {} games in {:.1}s, kept {}; target activity in kept games:", scouted, t0.elapsed().as_secs_f64(), chosen.len());
    for (k, v) in &total {
        eprintln!("  {:5}  {}", v, k);
    }
}
