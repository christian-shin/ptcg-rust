//! Run scenarios in the Rust engine alone (no oracle): play games from the
//! scenario's decks, apply its board edits at the first turn decision on or
//! after `scenario.turn`, follow its scripted `answers`, play the rest with
//! the oracle runner's policies, and check its `expect` assertions, the
//! invariants, stuck prompts, engine errors and panics (`ptcg::selfplay`).
//!
//!   scen <scenario.json|dir>... [--games N] [--seed S] [--threads T] [--full] [--record DIR] [--quiet]
//!
//! Game i of a scenario uses seed S * 100,000 + i, the decks paired from the
//! game index as the oracle's corpus command does, and policy heur (even i)
//! or random (odd i). `--record DIR` writes every game's trace to
//! DIR/<scenario>/g<seed>.json in the oracle's format (`diff` replays them).
//! Threads and priority as `fuzz` (half the cores at nice 10 on a Mac;
//! `--threads`, `--full`). Prints one line per scenario and a summary; exits 1 when any scenario has
//! a failure (an expect failure, an error, a stuck game, an invariant, a
//! panic, a scripted answer that is not among the options, or a scenario turn
//! that no game reached).

use ptcg::carddb::DefId;
use ptcg::selfplay::{expand_deck, load_scenario, play, End, Opts, Policy};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

fn arg<'a>(args: &'a [String], k: &str) -> Option<&'a str> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

#[derive(Default)]
struct Report {
    games: usize,
    failures: Vec<String>,
    broken: Vec<String>,
    not_reached: usize,
    exp_checked: usize,
    exp_failed: usize,
    exp_unchecked: usize,
}

fn run_scenario(path: &Path, games: usize, seed0: u32, record: Option<&Path>) -> Report {
    let mut rep = Report::default();
    let raw: Value = match std::fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) {
        Ok(v) => v,
        Err(e) => {
            rep.broken.push(format!("read: {}", e));
            return rep;
        }
    };
    let sc = match load_scenario(&raw) {
        Ok(v) => v,
        Err(e) => {
            rep.broken.push(e);
            return rep;
        }
    };
    let decks: Vec<Vec<DefId>> = match sc["decks"].as_array().map(|d| d.iter().map(expand_deck).collect::<Result<Vec<_>, _>>()) {
        Some(Ok(d)) if !d.is_empty() => d,
        Some(Err(e)) => {
            rep.broken.push(e);
            return rep;
        }
        _ => {
            rep.broken.push("scenario has no decks".into());
            return rep;
        }
    };
    let out_dir = record.map(|d| d.join(path.file_stem().unwrap()));
    if let Some(d) = &out_dir {
        std::fs::create_dir_all(d).unwrap();
    }
    for i in 0..games {
        let g = seed0.wrapping_mul(100_000).wrapping_add(i as u32);
        // Deterministic pairing from the game index, as the oracle's corpus command.
        let a = &decks[g as usize % decks.len()];
        let b = &decks[(g as usize / decks.len()) % decks.len()];
        let pol = if i % 2 == 0 { Policy::Heur } else { Policy::Random };
        let played = play(&Opts { seed: g, decks: [a, b], policy: [pol, pol], scenario: Some(&sc), record: out_dir.is_some() });
        if let (Some(d), Some(t)) = (&out_dir, &played.trace) {
            std::fs::write(d.join(format!("g{:06}.json", g)), serde_json::to_string(t).unwrap()).unwrap();
        }
        rep.games += 1;
        match played.end {
            End::Done { .. } => {
                if !played.reached {
                    rep.not_reached += 1;
                }
                if let Some(run) = played.expect {
                    if !run.failures.is_empty() {
                        rep.exp_failed += 1;
                        for f in &run.failures {
                            rep.failures.push(format!("game {}: EXPECT FAILED #{} {}\n    cite: {}\n    actual: {}", g, f.index, f.assertion, f.cite, f.actual));
                        }
                    } else if run.checked() == run.assertions.len() {
                        rep.exp_checked += 1;
                    } else {
                        rep.exp_unchecked += 1;
                    }
                }
            }
            End::Fail(e) => rep.failures.push(format!("game {}: {}", g, e)),
            End::Broken(e) => rep.broken.push(format!("game {}: {}", g, e)),
        }
    }
    rep
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let games: usize = arg(&args, "--games").map(|s| s.parse().expect("--games")).unwrap_or(8);
    let seed0: u32 = arg(&args, "--seed").map(|s| s.parse().expect("--seed")).unwrap_or(1);
    let threads = ptcg::selfplay::threads_from_args(&args);
    ptcg::selfplay::lower_priority(&args);
    let quiet = args.iter().any(|a| a == "--quiet");
    let record: Option<PathBuf> = arg(&args, "--record").map(PathBuf::from);
    let mut files: Vec<PathBuf> = Vec::new();
    let mut skip = false;
    for a in &args {
        if skip {
            skip = false;
            continue;
        }
        if ["--games", "--seed", "--threads", "--record"].contains(&a.as_str()) {
            skip = true;
            continue;
        }
        if a.starts_with("--") {
            continue;
        }
        let p = PathBuf::from(a);
        if p.is_dir() {
            let mut v: Vec<PathBuf> = std::fs::read_dir(&p).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map_or(false, |e| e == "json")).collect();
            v.sort();
            files.extend(v);
        } else {
            files.push(p);
        }
    }
    if files.is_empty() {
        eprintln!("usage: scen <scenario.json|dir>... [--games N] [--seed S] [--threads T] [--full] [--record DIR] [--quiet]");
        std::process::exit(2);
    }
    let t0 = std::time::Instant::now();
    let next = AtomicUsize::new(0);
    let reports: Mutex<Vec<Option<Report>>> = Mutex::new((0..files.len()).map(|_| None).collect());
    std::thread::scope(|s| {
        for _ in 0..threads.min(files.len()) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= files.len() {
                    break;
                }
                let r = run_scenario(&files[i], games, seed0, record.as_deref());
                reports.lock().unwrap()[i] = Some(r);
            });
        }
    });
    let reports = reports.into_inner().unwrap();
    let (mut bad, mut exp_bad, mut broken, mut games_total) = (0usize, 0usize, 0usize, 0usize);
    let (mut ec, mut ef, mut eu) = (0usize, 0usize, 0usize);
    for (f, r) in files.iter().zip(reports.iter()) {
        let r = r.as_ref().unwrap();
        games_total += r.games;
        ec += r.exp_checked;
        ef += r.exp_failed;
        eu += r.exp_unchecked;
        let not_clean = !r.failures.is_empty() || !r.broken.is_empty() || (r.games > 0 && r.not_reached == r.games);
        if r.exp_failed > 0 {
            exp_bad += 1;
        }
        if !r.broken.is_empty() || (r.games > 0 && r.not_reached == r.games) {
            broken += 1;
        }
        if not_clean {
            bad += 1;
        }
        if not_clean || !quiet {
            let mut line = format!("{} {}: {} games", if not_clean { "FAIL" } else { "ok  " }, f.display(), r.games);
            if r.exp_checked + r.exp_failed + r.exp_unchecked > 0 {
                line += &format!(", expect {} checked / {} failed / {} not checked", r.exp_checked, r.exp_failed, r.exp_unchecked);
            }
            if r.not_reached > 0 {
                line += &format!(", {} never reached the scenario turn", r.not_reached);
            }
            println!("{}", line);
            for l in r.broken.iter().take(3) {
                println!("  BROKEN {}", l);
            }
            for l in r.failures.iter().take(5) {
                println!("  {}", l);
            }
        }
    }
    println!(
        "\nscenarios: {} run ({} games, {:.1}s), {} not clean, {} with expect failures, {} broken; expect: {} games checked, {} failed, {} not checked",
        files.len(),
        games_total,
        t0.elapsed().as_secs_f64(),
        bad,
        exp_bad,
        broken,
        ec,
        ef,
        eu
    );
    ptcg::legal_stats::print_table();
    if bad > 0 {
        std::process::exit(1);
    }
}
