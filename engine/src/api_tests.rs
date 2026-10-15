//! The engine as a library (events batch 9 C13b): what a host that embeds it (a Python extension, a sandboxed agent) can
//! rely on. A game is plain data that is `Send + Clone`, and playing it makes no call into the operating system: no file,
//! no environment variable beyond the verification switches read once, no child process.

use crate::game::Game;
use crate::interface::SelectType;

fn assert_send_clone<T: Send + Clone>() {}

/// `Game` can be copied and moved to another thread (compile-time check).
#[test]
fn a_game_is_send_and_clone() {
    assert_send_clone::<Game>();
    assert_send_clone::<crate::state::State>();
    assert_send_clone::<crate::interface::SelectData>();
    assert_send_clone::<crate::view::TableView>();
}

fn source_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "bin") {
                continue;
            }
            source_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// No module outside `bin/` touches the file system, the process or the network, and the only environment reads are the
/// three switches below, each behind a `OnceLock` (read once per process, never on a decision path after the first):
/// - `game.rs`: `PTCG_VERIFY_CACHE` / `PTCG_VERIFY_LEGAL` (verification asserts);
/// - `legal_stats.rs`: `PTCG_LEGAL_STATS` (a counter switch);
/// - `options.rs`: `PTCG_VERIFY_LEGAL`.
///
/// A file's code is what comes before its unit-test module (the first `#[cfg(test)]` line).
#[test]
fn the_library_modules_do_no_io() {
    let mut files = Vec::new();
    source_files(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    assert!(files.len() > 100, "found {} source files", files.len());
    let banned = ["std::fs", "std::process", "std::net", "std::io::stdin", "std::thread::sleep", "Command::new", "File::open", "File::create"];
    let mut env_reads: Vec<String> = Vec::new();
    for f in &files {
        // The two files compiled only for `cargo test` (`#[cfg(test)] mod ...;` in lib.rs and spec/mod.rs).
        if f.file_name().is_some_and(|n| n == "api_tests.rs" || n == "usability_sweep.rs") {
            continue;
        }
        let text = std::fs::read_to_string(f).unwrap();
        let code = text.split("\n#[cfg(test)]").next().unwrap();
        let name = f.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap().display().to_string();
        for (i, line) in code.lines().enumerate() {
            let line = line.trim();
            if line.starts_with("//") {
                continue;
            }
            for b in banned {
                assert!(!line.contains(b), "{name}:{}: `{b}` outside bin/: {line}", i + 1);
            }
            // `use std::{fs, env}` and `use std::env;` forms.
            if let Some(rest) = line.strip_prefix("use std::") {
                let words: Vec<&str> = rest.split(|c: char| !c.is_alphanumeric() && c != '_').collect();
                for w in ["fs", "env", "process", "net"] {
                    assert!(!words.contains(&w), "{name}:{}: `use std::{w}` outside bin/: {line}", i + 1);
                }
            }
            if line.contains("std::env") || line.contains("env::var") || line.contains("env::args") {
                env_reads.push(format!("{name}: {line}"));
            }
        }
    }
    env_reads.sort();
    let allowed: Vec<&str> = vec!["src/game.rs", "src/legal_stats.rs", "src/options.rs"];
    let got: Vec<&str> = env_reads.iter().map(|r| r.split(':').next().unwrap()).collect();
    let got: Vec<String> = got.iter().map(|s| s.replace('\\', "/")).collect();
    assert_eq!(got, allowed, "environment reads outside bin/: {env_reads:#?}");
    for r in &env_reads {
        assert!(r.contains("get_or_init") && r.contains("std::env::var("), "an environment read must sit inside its OnceLock: {r}");
    }
}

/// `Game::new` and a thousand decisions on another thread (a finished game is followed by a new one): every decision
/// answered from the masks (as a host does), no panic, no I/O (the grep test above).
#[test]
fn a_thousand_decisions_run_on_another_thread() {
    let names: Vec<&str> = (0..60)
        .map(|i| match i {
            0..=3 => "Duraludon PRE 69",
            4..=6 => "Team Rocket's Mewtwo ex DRI 81",
            7..=10 => "Sacred Charm PFL 93",
            _ => "Metal Energy MEE 8",
        })
        .collect();
    let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
    let decisions = std::thread::spawn(move || {
        let mut n = 0usize;
        let mut seed = 11;
        while n < 1000 {
            // A finished game is followed by a new one.
            let mut g = Game::new(seed);
            seed += 1;
            g.start([&deck, &deck]).unwrap();
            while n < 1000 {
            let Some(sel) = g.select_with(false).unwrap() else { break };
            let mut picks = Vec::new();
            // A turn action: the last option (end the turn) keeps the games moving; a prompt: the first valid picks.
            if sel.select_type == SelectType::Main {
                picks.push(sel.options.len() - 1);
            } else {
                while picks.len() < sel.min_count {
                    let (mask, _) = g.pick_mask(&sel, &picks);
                    picks.push(mask.iter().position(|m| *m).expect("a valid pick"));
                }
            }
            g.answer(&sel, &picks).unwrap();
            n += 1;
            }
        }
        n
    })
    .join()
    .unwrap();
    assert_eq!(decisions, 1000, "decisions run");
}
