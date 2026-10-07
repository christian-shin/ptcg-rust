//! Rules audit (deck zone): plays random games with every pool card that
//! searches, looks at or shuffles the deck, and reports
//!   * a deck search (ChooseCards over the deck) that was not followed by a
//!     shuffle before the next turn action (Advanced Rulebook II-E-19, I-H);
//!   * a deck search for "any card" (no filter) whose minimum is below its maximum
//!     (Advanced Rulebook I-H, ruling 1778).
//!
//!   cargo run --profile iter --example audit_deck -- <games> [seed]
use ptcg::carddb::{cards, def, def_by_full_name, en_key, DefId};
use ptcg::game::{Action, Game, Pending};
use ptcg::interface::SelectType;
use ptcg::list::*;
use ptcg::options::legal_actions;
use ptcg::prompts::*;
use ptcg::rng::Rng;
use ptcg::state::ListRef;
use std::collections::BTreeMap;

fn filter_none(f: &Filter) -> bool {
    f.super_type.is_none() && f.stage.is_none() && f.trainer_type.is_none() && f.energy_type.is_none() && f.card_type.is_none() && f.tags.is_none() && f.name.is_none() && f.evolves_from.is_none()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let games: usize = args.first().map(|s| s.parse().unwrap()).unwrap_or(1000);
    let seed0: u32 = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(1);
    let want: Vec<String> = std::fs::read_to_string(std::env::var("TARGETS").unwrap_or("/tmp/audd/deckcards.txt".into())).unwrap().lines().map(|s| s.to_string()).collect();
    let targets: Vec<DefId> = want.iter().filter_map(|k| def_by_full_name(k)).collect();
    let basics: Vec<DefId> = (0..cards().len() as DefId).filter(|&i| def(i).is_pokemon() && def(i).stage == ptcg::types::Stage::Basic as u8 && def_by_full_name(en_key(i)).is_some()).collect();
    let energy = ["Fire Energy MEE", "Water Energy MEE", "Lightning Energy MEE", "Grass Energy MEE", "Psychic Energy MEE", "Darkness Energy MEE", "Fighting Energy MEE", "Metal Energy MEE"].map(|n| def_by_full_name(n).unwrap());
    eprintln!("{} targets, {} basics", targets.len(), basics.len());
    let mut rng = Rng::new(seed0);
    // per card: (searches, violations)
    let mut stat: BTreeMap<String, (u32, u32, u32)> = BTreeMap::new();
    let mut notes: BTreeMap<String, u32> = BTreeMap::new();
    for gi in 0..games {
        let mut decks: Vec<Vec<DefId>> = Vec::new();
        for _ in 0..2 {
            let mut d: Vec<DefId> = Vec::new();
            let n = 6 + rng.index(5);
            for _ in 0..n {
                let t = targets[rng.index(targets.len())];
                for _ in 0..(1 + rng.index(4)) {
                    if d.len() < 40 { d.push(t); }
                }
            }
            for _ in 0..3 {
                let b = basics[rng.index(basics.len())];
                for _ in 0..3 { d.push(b); }
            }
            let e = energy[rng.index(energy.len())];
            while d.len() < 60 { d.push(e); }
            d.truncate(60);
            decks.push(d);
        }
        let mut g = Game::new(seed0.wrapping_add(gi as u32 * 7919));
        if g.start([&decks[0], &decks[1]]).is_err() { continue; }
        let mut looked = [false; 2];
        let mut shuf = [false; 2];
        let mut prev_len = [60usize; 2];
        let mut last = String::from("setup");
        let mut last_search_msg = String::new();
        let mut steps = 0;
        loop {
            if steps > 4000 { break; }
            let sel = match g.select_with(true) { Ok(Some(s)) => s, _ => break };
            steps += 1;
            let cur_len = [g.st.players[0].deck.len(), g.st.players[1].deck.len()];
            let old_len = prev_len;
            prev_len = cur_len;
            match g.pending() {
                Pending::Chance(_) => {
                    if sel.select_type == SelectType::YesNo {
                        let k = rng.index(2);
                        if g.answer(&sel, &[k]).is_err() { break; }
                    } else {
                        let p = sel.player as usize;
                        looked[p] = false;
                        shuf[p] = true;
                        let n = g.st.players[p].deck.len();
                        let mut o: Vec<u8> = (0..n as u8).collect();
                        for i in (1..o.len()).rev() { o.swap(i, rng.index(i + 1)); }
                        if g.answer_shuffle(&sel, &o).is_err() { break; }
                    }
                    continue;
                }
                Pending::Turn(p) => {
                    shuf = [false; 2];
                    for q in 0..2 {
                        if looked[q] {
                            let e = stat.entry(last.clone()).or_default();
                            e.1 += 1;
                            *notes.entry(format!("NOSHUFFLE {} (prompt {})", last, last_search_msg)).or_default() += 1;
                            looked[q] = false;
                        }
                    }
                    let acts = legal_actions(&g);
                    // random answer
                    let n = sel.options.len();
                    let k = rng.index(n);
                    last = match acts.get(k).map(|o| o.action) {
                        Some(Action::PlayCard { hand_index, .. }) => g.st.players[p as usize].hand.get(hand_index as usize).map(|c| g.st.cdef(c).full_name.to_string()).unwrap_or("?".into()),
                        Some(Action::Attack { .. }) => g.st.active_pokemon(p as usize).map(|c| format!("attack of {}", g.st.cdef(c).full_name)).unwrap_or("?".into()),
                        Some(Action::UseAbility { name, .. }) | Some(Action::UseTrainerAbility { name, .. }) => format!("ability {}", name),
                        a => format!("{:?}", a),
                    };
                    if g.answer(&sel, &[k]).is_err() { break; }
                }
                Pending::Decision(i) => {
                    let pr = g.prompts.as_slice()[i];
                    if let PromptKind::ChooseCards { cards: ListRef::Temp(_), .. } = pr.kind {
                        for q in 0..2 {
                            if cur_len[q] < old_len[q] {
                                *notes.entry(format!("TOPLOOK {} (prompt {}) shuffled-already={}", last, pr.message, shuf[q])).or_default() += 1;
                                if !shuf[q] { looked[q] = true; last_search_msg = format!("TOPLOOK {}", pr.message); }
                            }
                        }
                    }
                    if let PromptKind::ChooseCards { cards: ListRef::Deck(p), filter, opts } = pr.kind {
                        if shuf[p as usize] { *notes.entry(format!("SHUFFLE-BEFORE-CHOOSE {} (prompt {})", last, pr.message)).or_default() += 1; } else { looked[p as usize] = true; }
                        last_search_msg = pr.message.to_string();
                        let e = stat.entry(last.clone()).or_default();
                        e.0 += 1;
                        let deck_n = g.st.players[p as usize].deck.len() as u8;
                        let nolim = opts.max_pokemons.is_none() && opts.max_basic_energies.is_none() && opts.max_energies.is_none() && opts.max_trainers.is_none() && opts.max_tools.is_none() && opts.max_stadiums.is_none() && opts.max_supporters.is_none() && opts.max_special_energies.is_none() && opts.max_items.is_none() && opts.max_basics.is_none() && opts.max_evolutions.is_none() && opts.max_stage1.is_none() && opts.max_stage2.is_none() && !opts.different_types && opts.blocked.0 == 0;
                        if filter_none(&filter) && nolim && opts.min < opts.max.min(deck_n) {
                            e.2 += 1;
                            *notes.entry(format!("ANYCARD-min {} (prompt {} min {} max {})", last, pr.message, opts.min, opts.max)).or_default() += 1;
                        }
                    }
                    let n = sel.options.len();
                    let mut done = false;
                    for _ in 0..30 {
                        let lo = sel.min_count.min(n);
                        let hi = sel.max_count.min(n);
                        let k = if hi > lo { lo + rng.index(hi - lo + 1) } else { lo };
                        let mut idx: Vec<usize> = (0..n).collect();
                        for i in (1..idx.len()).rev() { idx.swap(i, rng.index(i + 1)); }
                        idx.truncate(k);
                        idx.sort();
                        if g.answer(&sel, &idx).is_ok() { done = true; break; }
                    }
                    if !done { break; }
                }
                _ => break,
            }
        }
    }
    println!("--- per source (searches, noshuffle, anycard-min<max)");
    for (k, v) in &stat { if v.0 > 0 || v.1 > 0 { println!("{:60} {:5} {:4} {:4}", k, v.0, v.1, v.2); } }
    println!("--- notes");
    for (k, v) in &notes { println!("{:6} {}", v, k); }
}
