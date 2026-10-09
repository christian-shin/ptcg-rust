//! `PTCG_LEGAL_STATS=1`: how each legality answer was reached (by a rule
//! check or by a trial on a fork), per action kind, with the illegal trials
//! counted by error code. Thread-local counters merged into a global on
//! thread exit; the `fuzz` and `scen` binaries print the table at the end.

use std::collections::BTreeMap;
use std::sync::Mutex;

pub const KINDS: [&str; 12] = ["energy", "basic", "evolve", "item", "supporter", "stadium", "tool", "attack", "ability", "use_stadium", "retreat", "pass"];

pub fn enabled() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("PTCG_LEGAL_STATS").map_or(false, |v| v == "1"))
}

#[derive(Default, Clone)]
pub struct Stats {
    /// Per kind: [no-trial legal, no-trial illegal, trial legal, trial illegal].
    pub n: [[u64; 4]; 12],
    pub errs: BTreeMap<(usize, &'static str), u64>,
}

impl Stats {
    fn merge(&mut self, o: &Stats) {
        for k in 0..12 {
            for i in 0..4 {
                self.n[k][i] += o.n[k][i];
            }
        }
        for (key, v) in &o.errs {
            *self.errs.entry(*key).or_default() += v;
        }
    }
}

static GLOBAL: Mutex<Option<Stats>> = Mutex::new(None);

struct Local(Stats);

impl Drop for Local {
    fn drop(&mut self) {
        merge_into_global(&self.0);
    }
}

fn merge_into_global(s: &Stats) {
    let mut g = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
    g.get_or_insert_with(Stats::default).merge(s);
}

thread_local! {
    static LOCAL: std::cell::RefCell<Local> = std::cell::RefCell::new(Local(Stats::default()));
}

/// Record one answer: `trial` says how it was reached; `err` is the trial's error code when illegal.
pub fn record(kind: usize, trial: bool, legal: bool, err: Option<&'static str>) {
    LOCAL.with(|l| {
        let s = &mut l.borrow_mut().0;
        s.n[kind][(trial as usize) * 2 + (!legal) as usize] += 1;
        if let (true, false, Some(e)) = (trial, legal, err) {
            *s.errs.entry((kind, e)).or_default() += 1;
        }
    });
}

/// Print the table to stderr (when enabled), after flushing this thread's counters.
pub fn print_table() {
    if !enabled() {
        return;
    }
    LOCAL.with(|l| {
        let s = std::mem::take(&mut l.borrow_mut().0);
        merge_into_global(&s);
    });
    let g = GLOBAL.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_default();
    eprintln!("legality stats (answers per action kind)");
    eprintln!("{:<12} {:>10} {:>10} {:>10} {:>10}   {:>8}", "kind", "rule:legal", "rule:ill", "trial:leg", "trial:ill", "trial%");
    let mut tot = [0u64; 4];
    for (k, name) in KINDS.iter().enumerate() {
        let n = g.n[k];
        let all: u64 = n.iter().sum();
        if all == 0 {
            continue;
        }
        for i in 0..4 {
            tot[i] += n[i];
        }
        eprintln!("{:<12} {:>10} {:>10} {:>10} {:>10}   {:>7.1}%", name, n[0], n[1], n[2], n[3], 100.0 * (n[2] + n[3]) as f64 / all as f64);
    }
    let all: u64 = tot.iter().sum();
    eprintln!("{:<12} {:>10} {:>10} {:>10} {:>10}   {:>7.1}%", "total", tot[0], tot[1], tot[2], tot[3], 100.0 * (tot[2] + tot[3]) as f64 / all.max(1) as f64);
    eprintln!("illegal trials by error:");
    let mut v: Vec<_> = g.errs.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    for ((k, e), n) in v {
        eprintln!("  {:<12} {:<34} {:>10}", KINDS[*k], e, n);
    }
}
