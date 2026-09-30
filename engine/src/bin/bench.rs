//! Throughput benchmark (PLAN.md 5): random-play games/s, steps/s, and
//! state clone time.
//!
//!   bench <deckA.txt> <deckB.txt> [games]

use ptcg::carddb::def_by_full_name;
use ptcg::game::Game;
use ptcg::rng::Rng;
use std::time::Instant;

fn read_deck(path: &str) -> Vec<u16> {
    let text = std::fs::read_to_string(path).expect("deck file");
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let (n, name) = t.split_once(' ').unwrap();
        let d = def_by_full_name(name).unwrap_or_else(|| panic!("unknown card {}", name));
        for _ in 0..n.parse::<usize>().unwrap() {
            out.push(d);
        }
    }
    out
}

/// Play one game with uniformly random answers; returns decision steps.
pub fn random_game(decks: [&[u16]; 2], seed: u32, rng: &mut Rng) -> (usize, bool) {
    let mut g = Game::new(seed);
    g.start(decks).unwrap();
    let mut steps = 0;
    while let Ok(Some(sel)) = g.select() {
        if steps > 3000 {
            return (steps, false);
        }
        steps += 1;
        let n = sel.options.len();
        let mut done = false;
        for _ in 0..30 {
            let lo = sel.min_count.min(n);
            let hi = sel.max_count.min(n);
            let k = if hi > lo { lo + rng.index(hi - lo + 1) } else { lo };
            let mut idx: Vec<usize> = (0..n).collect();
            for i in (1..idx.len()).rev() {
                idx.swap(i, rng.index(i + 1));
            }
            idx.truncate(k);
            idx.sort();
            if g.answer(&sel, &idx).is_ok() {
                done = true;
                break;
            }
        }
        if !done {
            return (steps, false);
        }
    }
    (steps, true)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a = read_deck(&args[0]);
    let b = read_deck(&args[1]);
    let games: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(1000);
    println!("sizeof(Game) = {} bytes, sizeof(State) = {} bytes", std::mem::size_of::<Game>(), std::mem::size_of::<ptcg::state::State>());

    // Clone time on a mid-game state.
    let mut g = Game::new(7);
    g.start([&a, &b]).unwrap();
    let _ = g.select();
    let n = 200_000;
    let t = Instant::now();
    let mut acc = 0u64;
    for _ in 0..n {
        let c = std::hint::black_box(g);
        acc += c.st.turn as u64;
    }
    let clone_ns = t.elapsed().as_nanos() as f64 / n as f64;
    println!("clone: {:.1} ns ({})", clone_ns, acc % 2);

    let mut rng = Rng::new(12345);
    let t = Instant::now();
    let (mut steps, mut ok) = (0usize, 0usize);
    for i in 0..games {
        let (s, fin) = random_game([&a, &b], i as u32, &mut rng);
        steps += s;
        ok += fin as usize;
    }
    let dt = t.elapsed().as_secs_f64();
    println!(
        "{} games ({} finished) in {:.2}s: {:.1} games/s, {:.0} steps/s",
        games,
        ok,
        dt,
        games as f64 / dt,
        steps as f64 / dt
    );
}
