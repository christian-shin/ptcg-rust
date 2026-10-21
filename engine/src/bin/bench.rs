//! Throughput benchmark: uniformly random play from a fuzz spec's decks,
//! no JSON descriptors, no invariant checks, no recording.
//!
//!   bench [spec.json] [--games N] [--seed S] [--threads T] [--sizes] [--record] [--host [--apart]]
//!         [--search [--restores N] [--depth D] [--branch B]]
//!
//! `--search` measures what a search over copies of the game costs: the bench's games played at random, and at every turn
//! decision (a root) `N` restores (default 100: `copy_from` of the live game into a pooled game) timed alone, as many
//! whole-`Game` copies (`*dst = *src`), then a depth-limited DFS over the deciding player's own turn below the root
//! (default depth 3, up to `B` = 4 random actions of each node expanded): each node is a restore of its parent, one
//! step (the action, then random answers to the prompts up to the next turn decision) and the node's legal actions.
//! Prints restores/s, copies/s, the bytes a restore moves (`Game::copy_bytes`), DFS nodes/s.
//!
//! `--host` plays the same games the way a program that embeds the engine does (`ptcg::host`, its buffers kept between
//! decisions): at a turn decision the table as the deciding seat sees it and the legal actions in one call
//! (`decision_into`; with `--apart` as two, `view_into` and `legal_actions_into`), at a prompt the table (`view_into`), the
//! prompt's select (`select_into`) and pick masks (`pick_mask_into`); the all-or-nothing `act` / `answer`,
//! `settle`, the record if on (`take_record_into`); each call timed. It prints the microseconds per decision
//! of each phase after the single-thread run (the decision count is the same as without `--host`: the same choices).
//!
//! `--record` turns the game record on in every game (`Game::record_on`) and takes it after every decision: the cost of
//! the record (its entries are dropped).
//! `spec.json` is the fuzz deck spec (default `bench/spec.json`, a copy of the golden deck spec).
//! Game i has seed S * 100,000 + i and the decks pair as in `fuzz`. Turn
//! options come from the declared-checks fast path (`legal_actions`), prompt
//! answers pick uniformly from the select interface's pick masks. Prints
//! games/s, decisions/s and microseconds per decision on one thread, then the
//! same on 1, 2, 4, ... T threads (default: the cores) with the scaling factor (not with `--threads 1`).
//! The decision and winner counts are a determinism checksum: they must not
//! change between behavior-preserving commits.

use ptcg::carddb::DefId;
use ptcg::game::{Action, Game, Pending};
use ptcg::options::legal_actions_into;
use ptcg::rng::Rng;
use ptcg::selfplay::{expand_deck, MAX_STEPS, MAX_TURNS};
use serde_json::Value;
use std::time::Instant;

/// CPU time of the calling thread in seconds (wall time is noisy on a shared machine).
fn thread_cpu() -> f64 {
    #[repr(C)]
    struct Timespec {
        sec: i64,
        nsec: i64,
    }
    extern "C" {
        fn clock_gettime(clk: i32, ts: *mut Timespec) -> i32;
    }
    // CLOCK_THREAD_CPUTIME_ID: 16 on macOS, 3 on Linux.
    let clk = if cfg!(target_os = "macos") { 16 } else { 3 };
    let mut ts = Timespec { sec: 0, nsec: 0 };
    unsafe { clock_gettime(clk, &mut ts) };
    ts.sec as f64 + ts.nsec as f64 * 1e-9
}

fn arg<'a>(args: &'a [String], k: &str) -> Option<&'a str> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

#[derive(Default, Clone, Copy)]
struct Tally {
    games: u64,
    decisions: u64,
    finished: u64,
    wins0: u64,
    failed: u64,
}

impl Tally {
    fn add(&mut self, o: Tally) {
        self.games += o.games;
        self.decisions += o.decisions;
        self.finished += o.finished;
        self.wins0 += o.wins0;
        self.failed += o.failed;
    }
}

/// One game of uniformly random choices; returns (decisions, finished, winner, failed).
fn random_game(decks: [&[DefId]; 2], seed: u32) -> (u64, bool, i8, bool) {
    random_game_inner(decks, seed)
}

fn fail(decisions: u64, why: &str) -> (u64, bool, i8, bool) {
    if std::env::var("PTCG_BENCH_DEBUG").is_ok() {
        eprintln!("failed: {}", why);
    }
    (decisions, false, -1, true)
}

/// `--record`: every game records (set once, before the runs).
static RECORD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn random_game_inner(decks: [&[DefId]; 2], seed: u32) -> (u64, bool, i8, bool) {
    let mut g = Game::new(seed);
    let record = RECORD.load(std::sync::atomic::Ordering::Relaxed);
    if record {
        g.record_on();
    }
    let mut rng = Rng::new(seed.wrapping_mul(2654435761));
    if g.start(decks).and_then(|_| g.settle()).is_err() {
        return fail(0, "start");
    }
    let mut decisions = 0u64;
    let mut opts = Vec::new();
    let mut mask: Vec<bool> = Vec::new();
    for _ in 0..MAX_STEPS {
        if g.st.turn > MAX_TURNS {
            break;
        }
        match g.pending() {
            Pending::Finished => return (decisions, true, g.st.winner, false),
            Pending::Stuck => return fail(decisions, "stuck"),
            Pending::Chance(_) | Pending::Info(_) => {
                if g.settle().is_err() {
                    return fail(decisions, "settle");
                }
            }
            Pending::Turn(_) => {
                if record {
                    std::hint::black_box(g.take_record());
                }
                legal_actions_into(&g, &mut opts);
                if opts.is_empty() {
                    return fail(decisions, "no legal options");
                }
                decisions += 1;
                let k = rng.index(opts.len());
                if g.act_no_rollback(opts[k]).and_then(|_| g.settle()).is_err() {
                    return fail(decisions, "act");
                }
            }
            Pending::Decision(_) => {
                if record {
                    std::hint::black_box(g.take_record());
                }
                let Ok(Some(sel)) = g.select() else { return fail(decisions, "select") };
                decisions += 1;
                let n = sel.options.len();
                let mut picks: Vec<usize> = Vec::new();
                let mut ok = true;
                loop {
                    let stop = g.pick_mask_into(&sel, &picks, &mut mask);
                    let allowed: Vec<usize> = (0..n).filter(|j| mask[*j]).collect();
                    let choices = allowed.len() + stop as usize;
                    if choices == 0 {
                        ok = false;
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
                if !ok {
                    // Nothing can be chosen: decline (cancel) or answer empty if the prompt takes it.
                    let Pending::Decision(pi) = g.pending() else { return fail(decisions, "pending") };
                    let pr = g.prompts.as_slice()[pi];
                    let res = g.decline_answer(&pr).or_else(|_| g.decode_picks(&sel, &[])).ok();
                    let Some(res) = res else { return fail(decisions, "empty answer") };
                    if g.resolve(pi, res).and_then(|_| g.settle()).is_err() {
                        return fail(decisions, "empty answer");
                    }
                } else if g.answer_no_rollback(&sel, &picks).is_err() || g.settle().is_err() {
                    return fail(decisions, "answer");
                }
            }
        }
    }
    (decisions, false, -1, false)
}

/// The phases a host's decision is made of (`--host`), in the order of `PHASES`.
#[derive(Clone, Copy)]
enum Phase {
    Settle,
    View,
    Legal,
    Select,
    PickMask,
    Act,
    Answer,
    Record,
    /// The view and the legal actions of a turn decision in one call (`decision_into`).
    Decision,
}

const PHASES: [&str; 9] = ["settle", "view", "legal_actions", "select", "pick_mask", "act", "answer", "take_record", "view+legal"];

/// Nanoseconds and calls per phase of a `--host` run.
#[derive(Default, Clone, Copy)]
struct Clock {
    ns: [u64; 9],
    calls: [u64; 9],
}

impl Clock {
    #[inline]
    fn time<T>(&mut self, p: Phase, f: impl FnOnce() -> T) -> T {
        let t = Instant::now();
        let r = f();
        self.ns[p as usize] += t.elapsed().as_nanos() as u64;
        self.calls[p as usize] += 1;
        r
    }
}

/// [`random_game_inner`] the way a host plays it: the same choices from the same random stream (so the same games),
/// through `ptcg::host` with the buffers kept between decisions (the views, the action list, the masks, the record),
/// each call timed into `clock`; `apart`: a turn decision's view and actions read by two calls instead of one.
fn host_game(decks: [&[DefId]; 2], seed: u32, clock: &mut Clock, apart: bool) -> (u64, bool, i8, bool) {
    use ptcg::host::{self, TableView};
    let record = RECORD.load(std::sync::atomic::Ordering::Relaxed);
    let mut g = Game::new(seed);
    if record {
        g.record_on();
    }
    let mut rng = Rng::new(seed.wrapping_mul(2654435761));
    if g.start(decks).is_err() {
        return fail(0, "start");
    }
    // One view per seat, kept and refreshed at its seat's decisions.
    let mut views = [TableView::new(&g, 0), TableView::new(&g, 1)];
    let mut actions = Vec::new();
    let mut sel = host::SelectData::default();
    let mut mask = Vec::new();
    let mut entries = Vec::new();
    let mut picks: Vec<usize> = Vec::new();
    let mut allowed: Vec<usize> = Vec::new();
    let mut decisions = 0u64;
    for _ in 0..MAX_STEPS {
        if clock.time(Phase::Settle, || g.settle()).is_err() {
            return fail(decisions, "settle");
        }
        if g.st.turn > MAX_TURNS {
            break;
        }
        match g.pending() {
            Pending::Finished => return (decisions, true, g.st.winner, false),
            Pending::Stuck => return fail(decisions, "stuck"),
            Pending::Chance(_) | Pending::Info(_) => return fail(decisions, "settled"),
            Pending::Turn(seat) => {
                if record {
                    clock.time(Phase::Record, || host::take_record_into(&mut g, &mut entries));
                    std::hint::black_box(&entries);
                }
                if apart {
                    clock.time(Phase::View, || host::view_into(&g, seat as usize, &mut views[seat as usize]));
                    clock.time(Phase::Legal, || host::legal_actions_into(&g, &mut actions));
                } else {
                    clock.time(Phase::Decision, || host::decision_into(&g, seat as usize, &mut views[seat as usize], &mut actions));
                }
                std::hint::black_box(&views);
                if actions.is_empty() {
                    return fail(decisions, "no legal options");
                }
                decisions += 1;
                let k = rng.index(actions.len());
                if clock.time(Phase::Act, || g.act(actions[k])).is_err() {
                    return fail(decisions, "act");
                }
            }
            Pending::Decision(_) => {
                if record {
                    clock.time(Phase::Record, || host::take_record_into(&mut g, &mut entries));
                    std::hint::black_box(&entries);
                }
                let Ok(true) = clock.time(Phase::Select, || host::select_into(&mut g, &mut sel)) else { return fail(decisions, "select") };
                clock.time(Phase::View, || host::view_into(&g, sel.player as usize, &mut views[sel.player as usize]));
                std::hint::black_box(&views);
                decisions += 1;
                let n = sel.options.len();
                picks.clear();
                let mut ok = true;
                loop {
                    let stop = clock.time(Phase::PickMask, || host::pick_mask_into(&g, &sel, &picks, &mut mask));
                    allowed.clear();
                    allowed.extend((0..n).filter(|j| mask[*j]));
                    let choices = allowed.len() + stop as usize;
                    if choices == 0 {
                        ok = false;
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
                if !ok {
                    let Pending::Decision(pi) = g.pending() else { return fail(decisions, "pending") };
                    let pr = g.prompts.as_slice()[pi];
                    let res = g.decline_answer(&pr).or_else(|_| g.decode_picks(&sel, &[])).ok();
                    let Some(res) = res else { return fail(decisions, "empty answer") };
                    if g.resolve(pi, res).is_err() {
                        return fail(decisions, "empty answer");
                    }
                } else if clock.time(Phase::Answer, || g.answer(&sel, &picks)).is_err() {
                    return fail(decisions, "answer");
                }
            }
        }
    }
    (decisions, false, -1, false)
}

/// `--host`: the single-thread run through [`host_game`], with the phase table.
fn run_host(decks: &[Vec<DefId>], seed0: u32, games: usize, apart: bool) {
    let n = decks.len();
    let mut warm = Clock::default();
    for i in 0..20.min(games) {
        let g = seed0.wrapping_add(7777).wrapping_mul(100_000).wrapping_add(i as u32);
        host_game([&decks[g as usize % n], &decks[(g as usize / n) % n]], g, &mut warm, apart);
    }
    let mut clock = Clock::default();
    let mut t = Tally::default();
    let t0 = Instant::now();
    for i in 0..games {
        let g = seed0.wrapping_mul(100_000).wrapping_add(i as u32);
        let (a, b) = (g as usize % n, (g as usize / n) % n);
        let (d, fin, w, failed) = host_game([&decks[a], &decks[b]], g, &mut clock, apart);
        t.games += 1;
        t.decisions += d;
        t.finished += fin as u64;
        t.wins0 += (w == 0) as u64;
        t.failed += failed as u64;
    }
    let dt = t0.elapsed().as_secs_f64();
    let per = |ns: u64| ns as f64 * 1e-3 / t.decisions.max(1) as f64;
    println!(
        "host, 1 thread: {} games ({} finished, {} failed, p0 won {}) {} decisions in {:.2}s: {:.2} us/decision",
        t.games, t.finished, t.failed, t.wins0, t.decisions, dt, dt * 1e6 / t.decisions.max(1) as f64
    );
    let mut timed = 0;
    for (i, name) in PHASES.iter().enumerate() {
        timed += clock.ns[i];
        println!(
            "  {:<13} {:>7.2} us/decision ({:.2} calls/decision, {:.2} us/call)",
            name,
            per(clock.ns[i]),
            clock.calls[i] as f64 / t.decisions.max(1) as f64,
            clock.ns[i] as f64 * 1e-3 / clock.calls[i].max(1) as f64
        );
    }
    println!("  {:<13} {:>7.2} us/decision (the timed calls)", "sum", per(timed));
}

/// Buffers of the random play in `--search` (kept between decisions: no allocation per step).
#[derive(Default)]
struct Scratch {
    sel: ptcg::host::SelectData,
    mask: Vec<bool>,
    picks: Vec<usize>,
    allowed: Vec<usize>,
}

/// Plays on until the next turn decision or the end of the game: the chance prompts settled, every decision prompt
/// (either player's) answered by uniformly random picks from its pick masks, as the plain bench does.
fn play_to_turn(g: &mut Game, rng: &mut Rng, s: &mut Scratch) -> Result<(), &'static str> {
    for _ in 0..MAX_STEPS {
        match g.pending() {
            Pending::Finished | Pending::Turn(_) => return Ok(()),
            Pending::Stuck => return Err("stuck"),
            Pending::Chance(_) | Pending::Info(_) => g.settle().map_err(|_| "settle")?,
            Pending::Decision(pi) => {
                if !matches!(g.select_into(&mut s.sel), Ok(true)) {
                    return Err("select");
                }
                let n = s.sel.options.len();
                s.picks.clear();
                let mut ok = true;
                loop {
                    let stop = g.pick_mask_into(&s.sel, &s.picks, &mut s.mask);
                    s.allowed.clear();
                    s.allowed.extend((0..n).filter(|j| s.mask[*j]));
                    let choices = s.allowed.len() + stop as usize;
                    if choices == 0 {
                        ok = false;
                        break;
                    }
                    let r = rng.index(choices);
                    if r == s.allowed.len() {
                        break;
                    }
                    s.picks.push(s.allowed[r]);
                    if s.sel.max_count <= 1 {
                        break;
                    }
                }
                if !ok {
                    let pr = g.prompts.as_slice()[pi];
                    let res = g.decline_answer(&pr).or_else(|_| g.decode_picks(&s.sel, &[])).map_err(|_| "empty answer")?;
                    g.resolve(pi, res).map_err(|_| "empty answer")?;
                } else {
                    g.answer_no_rollback(&s.sel, &s.picks).map_err(|_| "answer")?;
                }
                g.settle().map_err(|_| "settle")?;
            }
        }
    }
    Err("steps")
}

/// One step of the search: the action, then the play to the next turn decision.
fn step(g: &mut Game, a: Action, rng: &mut Rng, s: &mut Scratch) -> Result<(), &'static str> {
    g.act_no_rollback(a).and_then(|_| g.settle()).map_err(|_| "act")?;
    play_to_turn(g, rng, s)
}

/// `--search` settings and counts.
#[derive(Default)]
struct Search {
    restores: usize,
    depth: usize,
    branch: usize,
    roots: u64,
    /// Restores timed alone (`copy_from` of the root into a pooled game), and their time.
    restored: u64,
    restore_ns: u64,
    /// The same number of whole-`Game` assignments (`*dst = *src`: `Game` is `Copy`), and their time.
    copy_ns: u64,
    /// Bytes per restore (`Game::copy_bytes`), summed over the roots.
    copied: u64,
    cleared: u64,
    copied_max: usize,
    /// DFS nodes (restore + step + legal actions), steps that failed, nodes at the own turn still (their legal actions
    /// listed) and the actions listed, and the DFS time.
    nodes: u64,
    failed: u64,
    own: u64,
    listed: u64,
    dfs_ns: u64,
}

/// The depth-limited DFS over the own turn of `p` below `pool[0]` (the node, whose legal actions are `acts[0]`): each
/// child is a restore of the node into `pool[1]`, one step, and the legal actions if it is `p`'s turn decision still
/// (expanded while `depth` remains). Up to `branch` actions of a node, chosen at random, are expanded.
fn dfs(pool: &mut [Box<Game>], p: u8, depth: usize, se: &mut Search, rng: &mut Rng, s: &mut Scratch, acts: &mut [Vec<Action>]) {
    let (node, below) = pool.split_first_mut().unwrap();
    let (opts, acts_below) = acts.split_first_mut().unwrap();
    let k = opts.len().min(se.branch);
    // A random `k` of the node's actions (a partial shuffle).
    for i in 0..k {
        let j = i + rng.index(opts.len() - i);
        opts.swap(i, j);
    }
    for i in 0..k {
        let child = &mut below[0];
        child.copy_from(node);
        se.nodes += 1;
        if step(child, opts[i], rng, s).is_err() {
            se.failed += 1;
            continue;
        }
        if child.pending() != Pending::Turn(p) {
            continue;
        }
        legal_actions_into(child, &mut acts_below[0]);
        se.own += 1;
        se.listed += acts_below[0].len() as u64;
        if depth > 1 && !acts_below[0].is_empty() {
            dfs(below, p, depth - 1, se, rng, s, acts_below);
        }
    }
}

/// `--search`: the bench's games played at random; at every turn decision (a root) `restores` restores of the live
/// game into a pooled game and as many whole-`Game` copies are timed, then the own-turn DFS below it.
fn run_search(decks: &[Vec<DefId>], seed0: u32, games: usize, se: &mut Search) {
    let n = decks.len();
    let depth = se.depth.max(1);
    let mut pool: Vec<Box<Game>> = (0..=depth).map(|_| Box::new(Game::new(0))).collect();
    let mut full = Box::new(Game::new(0));
    let mut acts: Vec<Vec<Action>> = (0..=depth).map(|_| Vec::new()).collect();
    let mut s = Scratch::default();
    let mut srng = Rng::new(seed0.wrapping_mul(40503));
    let mut t = Tally::default();
    let mut opts = Vec::new();
    for i in 0..games {
        let seed = seed0.wrapping_mul(100_000).wrapping_add(i as u32);
        let mut g = Game::new(seed);
        let mut rng = Rng::new(seed.wrapping_mul(2654435761));
        t.games += 1;
        if g.start([&decks[seed as usize % n], &decks[(seed as usize / n) % n]]).and_then(|_| g.settle()).is_err() {
            continue;
        }
        for _ in 0..MAX_STEPS {
            if g.st.turn > MAX_TURNS || play_to_turn(&mut g, &mut rng, &mut s).is_err() {
                break;
            }
            let Pending::Turn(p) = g.pending() else { break };
            legal_actions_into(&g, &mut opts);
            if opts.is_empty() {
                break;
            }
            se.roots += 1;
            // Restores: the live game into a pooled game, and as many whole-`Game` copies.
            let b = g.copy_bytes();
            se.copied += b.copied as u64;
            se.cleared += b.cleared as u64;
            se.copied_max = se.copied_max.max(b.copied);
            let t0 = Instant::now();
            for _ in 0..se.restores {
                pool[0].copy_from(std::hint::black_box(&g));
                std::hint::black_box(&mut pool[0]);
            }
            se.restore_ns += t0.elapsed().as_nanos() as u64;
            let t0 = Instant::now();
            for _ in 0..se.restores {
                **std::hint::black_box(&mut full) = *std::hint::black_box(&g);
            }
            se.copy_ns += t0.elapsed().as_nanos() as u64;
            se.restored += se.restores as u64;
            // The DFS below the root.
            let t0 = Instant::now();
            pool[0].copy_from(&g);
            acts[0].clone_from(&opts);
            dfs(&mut pool, p, depth, se, &mut srng, &mut s, &mut acts);
            se.dfs_ns += t0.elapsed().as_nanos() as u64;
            let k = rng.index(opts.len());
            if step(&mut g, opts[k], &mut rng, &mut s).is_err() {
                break;
            }
        }
        if g.pending() == Pending::Finished {
            t.finished += 1;
            t.wins0 += (g.st.winner == 0) as u64;
        }
    }
    let per = |ns: u64, k: u64| ns as f64 * 1e-3 / k.max(1) as f64;
    let rate = |ns: u64, k: u64| k as f64 / (ns as f64 * 1e-9).max(1e-9);
    let roots = se.roots.max(1);
    println!(
        "search: {} games ({} finished, p0 won {}), {} roots (turn decisions); {} restores per root; DFS over the own turn, depth {}, branch {}",
        t.games, t.finished, t.wins0, se.roots, se.restores, depth, se.branch
    );
    println!(
        "  restore (copy_from into a pooled game): {:.0} restores/s, {:.3} us/restore; bytes copied {:.0} mean, {} max, + {} cleared (derived stamps)",
        rate(se.restore_ns, se.restored),
        per(se.restore_ns, se.restored),
        se.copied as f64 / roots as f64,
        se.copied_max,
        se.cleared / roots
    );
    println!(
        "  whole-Game copy (*dst = *src, {} B): {:.0} copies/s, {:.3} us/copy",
        std::mem::size_of::<Game>(),
        rate(se.copy_ns, se.restored),
        per(se.copy_ns, se.restored)
    );
    println!(
        "  DFS: {} nodes ({} failed steps; {} at the own turn, {:.1} legal actions each) in {:.2}s: {:.0} nodes/s, {:.3} us/node (restore + step + legal actions)",
        se.nodes,
        se.failed,
        se.own,
        se.listed as f64 / se.own.max(1) as f64,
        se.dfs_ns as f64 * 1e-9,
        rate(se.dfs_ns, se.nodes),
        per(se.dfs_ns, se.nodes)
    );
}

fn run(decks: &[Vec<DefId>], seed0: u32, range: std::ops::Range<usize>) -> Tally {
    let n = decks.len();
    let mut t = Tally::default();
    for i in range {
        let g = seed0.wrapping_mul(100_000).wrapping_add(i as u32);
        let (a, b) = (g as usize % n, (g as usize / n) % n);
        let (d, fin, w, failed) = random_game([&decks[a], &decks[b]], g);
        t.games += 1;
        t.decisions += d;
        t.finished += fin as u64;
        t.wins0 += (w == 0) as u64;
        t.failed += failed as u64;
    }
    t
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let spec_path = args.first().filter(|a| !a.starts_with("--")).cloned().unwrap_or_else(|| "bench/spec.json".into());
    let games: usize = arg(&args, "--games").map(|s| s.parse().expect("--games")).unwrap_or(300);
    let seed0: u32 = arg(&args, "--seed").map(|s| s.parse().expect("--seed")).unwrap_or(11);
    let max_threads: usize = arg(&args, "--threads").map(|s| s.parse().expect("--threads")).unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()));
    if args.iter().any(|a| a == "--sizes") {
        use std::mem::size_of;
        println!(
            "sizeof: Game {} B, State {} B, Player {} B, Slot {} B, CardInst {} B, Effect {} B, PromptRec {} B, TableView {} B, Ctx {} B",
            size_of::<Game>(),
            size_of::<ptcg::state::State>(),
            size_of::<ptcg::state::Player>(),
            size_of::<ptcg::state::Slot>(),
            size_of::<ptcg::state::CardInst>(),
            size_of::<ptcg::effects::Effect>(),
            size_of::<ptcg::prompts::PromptRec>(),
            size_of::<ptcg::view::TableView>(),
            size_of::<ptcg::legal::Ctx>()
        );
    }
    RECORD.store(args.iter().any(|a| a == "--record"), std::sync::atomic::Ordering::Relaxed);
    let spec: Value = serde_json::from_str(&std::fs::read_to_string(&spec_path).expect("spec")).expect("spec json");
    let mut decks: Vec<Vec<DefId>> = Vec::new();
    for d in spec["decks"].as_array().expect("spec.decks") {
        decks.push(expand_deck(&d["cards"]).unwrap_or_else(|e| panic!("deck {}: {}", d["name"], e)));
    }

    if args.iter().any(|a| a == "--host") {
        run_host(&decks, seed0, games, args.iter().any(|a| a == "--apart"));
        return;
    }
    if args.iter().any(|a| a == "--search") {
        let num = |k: &str, d: usize| arg(&args, k).map(|s| s.parse().expect(k)).unwrap_or(d);
        let mut se = Search { restores: num("--restores", 100), depth: num("--depth", 3), branch: num("--branch", 4), ..Search::default() };
        run_search(&decks, seed0, games, &mut se);
        return;
    }
    // Warm up (card tables, caches), then the timed single-thread run.
    run(&decks, seed0.wrapping_add(7777), 0..20.min(games));
    let (t0, c0) = (Instant::now(), thread_cpu());
    let t = run(&decks, seed0, 0..games);
    let (dt, cpu) = (t0.elapsed().as_secs_f64(), thread_cpu() - c0);
    println!(
        "1 thread: {} games ({} finished, {} failed, p0 won {}) {} decisions in {:.2}s wall / {:.2}s cpu: {:.1} games/s, {:.0} decisions/s, {:.2} us/decision (cpu {:.2})",
        t.games, t.finished, t.failed, t.wins0, t.decisions, dt, cpu, t.games as f64 / dt, t.decisions as f64 / dt, dt * 1e6 / t.decisions.max(1) as f64, cpu * 1e6 / t.decisions.max(1) as f64
    );
    // `--threads 1`: the timed run is all (the scaling run on one thread would repeat it; tools/instructions.py counts
    // the instructions of exactly these games).
    if max_threads <= 1 {
        return;
    }

    let mut base = 0.0;
    let mut n = 1;
    loop {
        let t0 = Instant::now();
        let mut tot = Tally::default();
        std::thread::scope(|s| {
            let hs: Vec<_> = (0..n)
                .map(|w| {
                    let decks = &decks;
                    s.spawn(move || run(decks, seed0.wrapping_add(w as u32 * 1000), 0..games))
                })
                .collect();
            for h in hs {
                tot.add(h.join().unwrap());
            }
        });
        let dt = t0.elapsed().as_secs_f64();
        let dps = tot.decisions as f64 / dt;
        if n == 1 {
            base = dps;
        }
        println!("threads {:>2}: {:>7.1} games/s, {:>9.0} decisions/s, {:.2}x", n, tot.games as f64 / dt, dps, dps / base);
        if n >= max_threads {
            break;
        }
        n = (n * 2).min(max_threads);
    }
}
