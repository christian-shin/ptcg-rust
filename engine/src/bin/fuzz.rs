//! Fresh-seed self-play in the Rust engine alone (the oracle-free tier 4):
//! play games from a deck spec and fail on engine errors, stuck prompts,
//! turns without legal options, invariant violations and panics
//! (`ptcg::selfplay`).
//!
//!   fuzz <spec.json> [--games N] [--seed S] [--start I] [--threads T] [--full] [--out DIR] [--keep]
//!
//! `spec.json` is the corpus spec `{"decks": [{name, cards}], "policies": [...]}`
//! (`tools/fuzz.py` builds it like tier 4: meta decks plus random legal pool
//! decks). Game index i (from I) has seed S * 100,000 + i; the decks pair and
//! the policy is chosen from the seed as the oracle's corpus command does.
//! With `--out`, a failing game is played again with recording on and its
//! trace written to DIR/g<seed>.json (`diff` replays it); `--keep` records
//! every game and writes every trace (a corpus). DIR/summary.json holds the
//! counts. Exits 1 on any failure.
//!
//! On a Mac it uses half the cores at low priority (nice 10) by default;
//! elsewhere (the box) every core. `--threads T` sets the count, `--full`
//! uses every core at normal priority.

use ptcg::carddb::DefId;
use ptcg::selfplay::{expand_deck, play, End, Opts, Policy};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

fn arg<'a>(args: &'a [String], k: &str) -> Option<&'a str> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

#[derive(Default)]
struct Totals {
    games: usize,
    finished: usize,
    cap: usize,
    turns: u64,
    failures: Vec<(u32, String, String)>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(spec_path) = args.first().filter(|a| !a.starts_with("--")) else {
        eprintln!("usage: fuzz <spec.json> [--games N] [--seed S] [--start I] [--threads T] [--full] [--out DIR] [--keep]");
        std::process::exit(2);
    };
    let games: usize = arg(&args, "--games").map(|s| s.parse().expect("--games")).unwrap_or(1000);
    let seed0: u32 = arg(&args, "--seed").map(|s| s.parse().expect("--seed")).unwrap_or(1000);
    let start: u32 = arg(&args, "--start").map(|s| s.parse().expect("--start")).unwrap_or(0);
    let threads = ptcg::selfplay::threads_from_args(&args);
    ptcg::selfplay::lower_priority(&args);
    let out: Option<PathBuf> = arg(&args, "--out").map(PathBuf::from);
    let keep = args.iter().any(|a| a == "--keep");
    if let Some(d) = &out {
        std::fs::create_dir_all(d).unwrap();
    }
    let spec: Value = serde_json::from_str(&std::fs::read_to_string(spec_path).expect("spec")).expect("spec json");
    let mut names: Vec<String> = Vec::new();
    let mut decks: Vec<Vec<DefId>> = Vec::new();
    for d in spec["decks"].as_array().expect("spec.decks") {
        names.push(d["name"].as_str().unwrap_or("").to_string());
        match expand_deck(&d["cards"]) {
            Ok(v) => decks.push(v),
            Err(e) => {
                eprintln!("deck {}: {}", d["name"], e);
                std::process::exit(2);
            }
        }
    }
    let pols: Vec<Policy> = spec["policies"].as_array().map(|p| p.iter().filter_map(|x| x.as_str()).map(Policy::parse).collect()).filter(|p: &Vec<Policy>| !p.is_empty()).unwrap_or_else(|| vec![Policy::Heur, Policy::Random]);
    let n = decks.len();
    let seed_of = |i: u32| seed0.wrapping_mul(100_000).wrapping_add(i);
    let setup = |g: u32| -> (usize, usize, Policy) {
        // The oracle's corpus command: pairing and policy from the game seed.
        (g as usize % n, (g as usize / n) % n, pols[g as usize % pols.len()])
    };
    let t0 = std::time::Instant::now();
    let next = AtomicUsize::new(0);
    let totals = Mutex::new(Totals::default());
    std::thread::scope(|s| {
        for _ in 0..threads.max(1) {
            s.spawn(|| {
                let mut local = Totals::default();
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= games {
                        break;
                    }
                    let g = seed_of(start + i as u32);
                    let (a, b, pol) = setup(g);
                    let opts = Opts { seed: g, decks: [&decks[a], &decks[b]], policy: [pol, pol], scenario: None, record: keep && out.is_some() };
                    let played = play(&opts);
                    local.games += 1;
                    local.turns += played.turn.max(0) as u64;
                    let failed = match &played.end {
                        End::Done { status } => {
                            if *status == "cap" {
                                local.cap += 1;
                            } else {
                                local.finished += 1;
                            }
                            None
                        }
                        End::Fail(e) | End::Broken(e) => Some(e.clone()),
                    };
                    let trace = match (&failed, &out, played.trace) {
                        (_, _, Some(t)) => Some(t),
                        // Same seed, same game: play the failing one again to record it.
                        (Some(_), Some(_), None) => play(&Opts { record: true, ..opts }).trace,
                        _ => None,
                    };
                    if let (Some(d), Some(t)) = (&out, &trace) {
                        std::fs::write(d.join(format!("g{:010}.json", g)), serde_json::to_string(t).unwrap()).unwrap();
                    }
                    if let Some(e) = failed {
                        local.failures.push((g, format!("{} vs {} {}", names[a], names[b], pol.name()), e));
                    }
                }
                let mut t = totals.lock().unwrap();
                t.games += local.games;
                t.finished += local.finished;
                t.cap += local.cap;
                t.turns += local.turns;
                t.failures.extend(local.failures);
            });
        }
    });
    let mut t = totals.into_inner().unwrap();
    t.failures.sort_by_key(|f| f.0);
    let secs = t0.elapsed().as_secs_f64();
    // Group failures by their message without the step and turn numbers.
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for (g, setup, e) in &t.failures {
        println!("FAIL g{} ({}): {}", g, setup, e.lines().next().unwrap_or(""));
        let key: String = e.split(" at step").next().unwrap_or(e).chars().take(120).collect();
        *kinds.entry(key).or_default() += 1;
    }
    println!(
        "\nfuzz: {} games in {:.1}s ({:.0} games/s, {} threads): {} finished, {} at the cap, {} failed; {:.1} turns/game",
        t.games,
        secs,
        t.games as f64 / secs.max(1e-9),
        threads,
        t.finished,
        t.cap,
        t.failures.len(),
        t.turns as f64 / t.games.max(1) as f64
    );
    for (k, v) in &kinds {
        println!("  {:5}  {}", v, k);
    }
    if let Some(d) = &out {
        let summary = json!({
            "games": t.games, "finished": t.finished, "cap": t.cap, "failed": t.failures.len(),
            "seed": seed0, "start": start, "seconds": secs,
            "failures": t.failures.iter().map(|(g, s, e)| json!({ "seed": g, "setup": s, "error": e })).collect::<Vec<_>>(),
        });
        std::fs::write(d.join("summary.json"), serde_json::to_string_pretty(&summary).unwrap()).unwrap();
    }
    ptcg::legal_stats::print_table();
    if !t.failures.is_empty() {
        std::process::exit(1);
    }
}
