//! The engine as a library (events batch 9 C13b): what a host that embeds it (a Python extension, a sandboxed agent) can
//! rely on. A game is plain data that is `Send + Clone`, and playing it makes no call into the operating system: no file,
//! no environment variable beyond the verification switches read once, no child process.

use crate::game::Game;
use crate::interface::SelectType;
use crate::list::CardList;

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

/// Ends the turn of the player whose turn it is and plays the others' turns by ending them (prompts answered with the first
/// valid picks), until that player's next turn decision.
fn to_next_own_turn(g: &mut Game) {
    let me = g.st.active_player;
    let mut ended = false;
    while let Some(sel) = g.select_with(false).unwrap() {
        if sel.select_type == SelectType::Main {
            if sel.player == me && ended {
                return;
            }
            ended |= sel.player == me;
            let end = sel.options.len() - 1;
            g.answer(&sel, &[end]).unwrap();
            continue;
        }
        let mut picks = Vec::new();
        while picks.len() < sel.min_count {
            let (mask, _) = g.pick_mask(&sel, &picks);
            picks.push(mask.iter().position(|m| *m).expect("a valid pick"));
        }
        g.answer(&sel, &picks).unwrap();
    }
    panic!("the game ended");
}

/// The purpose of the decision the game waits for.
fn asked(g: &Game) -> crate::purpose::PromptPurpose {
    let crate::game::Pending::Decision(i) = g.pending() else { panic!("no prompt decision") };
    g.prompts.as_slice()[i].purpose
}

/// Plays the card named `name` from the hand of the player whose turn it is (its first legal play).
fn play(g: &mut Game, name: &str) -> crate::list::CardId {
    let p = g.st.active_player as usize;
    let k = g.st.players[p].hand.iter().position(|c| g.st.cdef(c).name == name).unwrap();
    let card = g.st.players[p].hand.as_slice()[k];
    let action = crate::options::legal_actions(g).into_iter().map(|o| o.action).find(|a| matches!(a, crate::game::Action::PlayCard { hand_index, .. } if *hand_index as usize == k)).unwrap();
    g.act(action).unwrap();
    card
}

/// A prompt says which card's effect asks and which card the question is about (events batch 10 E1): a Trainer's question
/// is its own, a rules question (setting up) is nobody's.
#[test]
fn a_trainer_asks_its_question() {
    use crate::prompts::Provenance;
    const MON: &str = "Duraludon PRE 69";
    let cards = [(MON, 4), ("Switch 30C 127", 4)];
    let mut g = Game::new(5);
    let deck: Vec<u16> = cards.iter().flat_map(|(n, k)| std::iter::repeat_n(*n, *k)).chain(std::iter::repeat("Metal Energy MEE 8")).take(60).map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
    g.start([&deck, &deck]).unwrap();
    let setup = g.select().unwrap().unwrap();
    assert_eq!(setup.provenance, Provenance::NONE, "setting up is a rules question");
    let mut g = crate::place::tests::game_of(&cards, serde_json::json!({"me": {"reset": true, "active": MON, "bench": [{"card": MON}], "hand": ["Switch 30C 127"]}, "opp": {"reset": true, "active": MON}}));
    let switch = play(&mut g, "Switch");
    let sel = g.select().unwrap().unwrap();
    assert_eq!(asked(&g), crate::purpose::PromptPurpose::SwitchPokemon);
    assert_eq!(sel.provenance, Provenance::by(switch));
}

/// Rare Candy's two questions: which Pokémon (nothing chosen yet), then which Stage 2 card, about the Pokémon chosen.
#[test]
fn an_evolution_asks_about_its_other_half() {
    use crate::prompts::Provenance;
    let cards = [("Dreepy ASC 158", 4), ("Dragapult ex ASC 160", 2), ("Rare Candy MEG 125", 4)];
    let mut g = crate::place::tests::game_of(&cards, serde_json::json!({"me": {"reset": true, "active": "Dreepy ASC 158"}, "opp": {"reset": true, "active": "Dreepy ASC 158"}}));
    to_next_own_turn(&mut g);
    crate::scenario::apply(&mut g, &serde_json::json!({"me": {"reset": true, "active": "Dreepy ASC 158", "hand": ["Rare Candy MEG 125", "Dragapult ex ASC 160"]}, "opp": {"reset": true, "active": "Dreepy ASC 158"}})).unwrap();
    let p = g.st.active_player as usize;
    let dreepy = g.st.active_pokemon(p).unwrap();
    let candy = play(&mut g, "Rare Candy");
    let sel = g.select().unwrap().unwrap();
    assert_eq!(asked(&g), crate::purpose::PromptPurpose::EvolvePokemon);
    assert_eq!(sel.provenance, Provenance::by(candy));
    let k = sel.options.iter().position(|o| o.serial == Some(dreepy)).unwrap();
    g.answer(&sel, &[k]).unwrap();
    let sel = g.select().unwrap().unwrap();
    assert_eq!(asked(&g), crate::purpose::PromptPurpose::EvolutionCard);
    assert_eq!(sel.provenance, Provenance::by(candy).about(dreepy));
}

/// An Ability's "you may" is asked by the Pokémon, about the Pokémon whose Ability may be used (Kadabra's Psychic Draw when
/// it is played to evolve).
#[test]
fn a_you_may_asks_about_its_card() {
    use crate::prompts::Provenance;
    let cards = [("Abra MEG 54", 4), ("Kadabra MEG 55", 4)];
    let mut g = crate::place::tests::game_of(&cards, serde_json::json!({"me": {"reset": true, "active": "Abra MEG 54"}, "opp": {"reset": true, "active": "Abra MEG 54"}}));
    to_next_own_turn(&mut g);
    crate::scenario::apply(&mut g, &serde_json::json!({"me": {"reset": true, "active": "Abra MEG 54", "hand": ["Kadabra MEG 55"]}, "opp": {"reset": true, "active": "Abra MEG 54"}})).unwrap();
    let kadabra = play(&mut g, "Kadabra");
    let sel = g.select().unwrap().unwrap();
    assert_eq!(asked(&g), crate::purpose::PromptPurpose::Activate);
    assert_eq!(sel.provenance, Provenance::by(kadabra).about(kadabra));
}

/// The host facade (`ptcg::host`): its decision and view types move between threads, and a game started through it plays
/// a few hundred decisions on another thread with no other call.
#[test]
fn the_host_facade_runs_on_another_thread() {
    use crate::host::*;
    assert_send_clone::<Decision>();
    assert_send_clone::<RecordEntry>();
    assert_send_clone::<CardPlace>();
    assert_send_clone::<StartOptions>();
    let deck: Vec<DefId> = (0..60)
        .map(|i| card_by_name(match i { 0..=3 => "Duraludon PRE 69", 4..=6 => "Team Rocket's Mewtwo ex DRI 81", 7..=10 => "Sacred Charm PFL 93", _ => "Metal Energy MEE 8" }).unwrap())
        .collect();
    let n = std::thread::spawn(move || {
        let mut g = start_game([&deck, &deck], 4, &StartOptions::default()).unwrap();
        let mut n = 0;
        while n < 300 {
            match next(&mut g).unwrap() {
                Decision::Over { .. } => break,
                Decision::Turn { actions, .. } => act(&mut g, *actions.last().unwrap()).unwrap(),
                Decision::Prompt { select, .. } => {
                    let mut picks = Vec::new();
                    while picks.len() < select.min_count {
                        let (mask, _) = pick_mask(&g, &select, &picks);
                        picks.push(mask.iter().position(|m| *m).expect("a valid pick"));
                    }
                    answer(&mut g, &select, &picks).unwrap();
                }
            }
            n += 1;
        }
        n
    })
    .join()
    .unwrap();
    assert!(n > 20);
}
