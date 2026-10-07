//! Playability probe (rules audit): for every Trainer card (and every Pokemon with
//! an Ability) of the pool, build crafted positions in the Rust engine and print
//! whether the card / its Ability is offered as a legal turn option there.
//! The audit then checks each offer against the rule "a Trainer or Ability that
//! would not change the game state in any way can't be used" (Advanced Rulebook
//! A-02, B-01, B-03, B-04, D-04, D-05, E-30).
//!
//!   probe [--cards "Name A|Name B"]       (default: every pool Trainer and Ability Pokemon)
//!
//! One line per card and position: `card | position | offered`.
//! Positions: A barren (1-card hand, no Bench, empty discard), B 4-card hand,
//! C = B with an empty deck, D = B with a stocked discard, E = D with damage,
//! Energy and a Tool on the boards and Benched Pokemon, F = E with full Benches.

use ptcg::carddb::{def, def_by_full_name, DefId};
use ptcg::game::{Action, Game, Pending};
use ptcg::list::*;
use ptcg::options::legal_turn_options;
use ptcg::state::ListRef;
use ptcg::types::*;
use serde_json::{json, Value};

const FILLER: &str = "Minccino JTG 125";
const ENERGY: &str = "Water Energy MEE";

fn fname(n: &str) -> &'static str {
    def(def_by_full_name(n).unwrap_or_else(|| panic!("unknown card {}", n))).full_name
}

fn reach_turn(g: &mut Game) -> bool {
    for _ in 0..2000 {
        match g.pending() {
            Pending::Turn(_) => {
                if g.st.turn >= 2 {
                    return true;
                }
                if g.act(Action::Pass).and_then(|_| g.settle()).is_err() {
                    return false;
                }
            }
            Pending::Decision(pi) => {
                let sel = match g.select() {
                    Ok(Some(s)) => s,
                    _ => return false,
                };
                let (mask, _) = g.pick_mask(&sel, &[]);
                let mut picks: Vec<usize> = vec![];
                if let Some(j) = (0..sel.options.len()).find(|j| mask[*j]) {
                    picks.push(j);
                }
                let wire = match sel.wire_answer(&picks) {
                    Ok(w) => w,
                    Err(_) => return false,
                };
                let pr = g.prompts.as_slice()[pi];
                let res = match g.decode_answer(&pr, &wire) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("decode {:?}", e);
                        return false;
                    }
                };
                if g.resolve(pi, res).and_then(|_| g.settle()).is_err() {
                    return false;
                }
            }
            Pending::Finished | Pending::Stuck => {
                eprintln!("reach_turn: finished/stuck at turn {} phase {:?}", g.st.turn, g.st.phase);
                return false;
            }
            _ => {
                if g.settle().is_err() {
                    return false;
                }
            }
        }
    }
    false
}

/// Evolution line of a Pokemon card, Basic first (cards named by `evolves_from`).
fn stack_for(t: DefId) -> Vec<DefId> {
    let mut v = vec![t];
    loop {
        let top = def(v[0]);
        if top.evolves_from.is_empty() || top.stage == Stage::Basic as u8 {
            break;
        }
        let cand = ptcg::carddb::cards().iter().enumerate().find(|(_, c)| c.name == top.evolves_from && c.is_pokemon());
        match cand {
            Some((i, _)) => v.insert(0, i as DefId),
            None => break,
        }
    }
    v
}

fn names_of(g: &Game, l: &[u8], skip: Option<u8>) -> String {
    let mut v: Vec<String> = l.iter().filter(|c| Some(**c) != skip).map(|c| g.st.cdef(*c).full_name.to_string()).collect();
    v.sort();
    v.join(",")
}

/// Everything a rule could call "the game state", except the order of the decks and the played card.
fn sig(g: &Game, skip: Option<u8>, markers: bool) -> String {
    let mut o = String::new();
    for p in 0..2 {
        let pl = &g.st.players[p];
        o += &format!("P{} hand[{}] deck[{}] disc[{}] lost[{}] stad[{}] ", p, names_of(g, pl.hand.as_slice(), skip), names_of(g, pl.deck.as_slice(), skip), names_of(g, pl.discard.as_slice(), skip), names_of(g, pl.lostzone.as_slice(), skip), names_of(g, pl.stadium.as_slice(), None));
        for i in 0..pl.prize_count as usize {
            o += &format!("prize{}[{}] ", i, names_of(g, pl.prizes[i].as_slice(), skip));
        }
        let mut slots = vec![pl.active];
        slots.extend(pl.bench.iter().copied());
        for (k, s) in slots.iter().enumerate() {
            let sl = &pl.slots[*s as usize];
            o += &format!("S{}[{:?}|{}|{}|d{}|{:?}|m{}] ", k, sl.cards.as_slice(), names_of(g, sl.energies.as_slice(), None), names_of(g, sl.tools.as_slice(), None), sl.damage, sl.special_conditions.as_slice(), if markers { sl.marker.items.len() } else { 0 });
        }
        o += &format!("pm{} ", if markers { pl.marker.items.len() } else { 0 });
    }
    o
}

/// Depth-first over the choices of an option (every single pick, a minimal and a maximal multi-pick, both
/// coin outcomes): true as soon as some outcome changes the state signature.
fn explore(g: &mut Game, before: &str, skip: Option<u8>, markers: bool, budget: &mut usize, stuck: &mut bool, depth: usize) -> bool {
    if *budget == 0 || depth > 6 {
        return true;
    }
    let pi = match g.pending() {
        Pending::Decision(pi) => pi,
        _ => return sig(g, skip, markers) != before,
    };
    let sel = match g.select() {
        Ok(Some(s)) => s,
        _ => {
            *stuck = true;
            return false;
        }
    };
    let mut answers: Vec<Vec<usize>> = vec![];
    if sel.max_count <= 1 {
        let (mask, stop) = g.pick_mask(&sel, &[]);
        for j in 0..sel.options.len() {
            if mask[j] && answers.len() < 8 {
                answers.push(vec![j]);
            }
        }
        if stop {
            answers.push(vec![]);
        }
    } else {
        for maximal in [false, true] {
            for last in [false, true] {
                let mut picks: Vec<usize> = vec![];
                loop {
                    let (mask, stop) = g.pick_mask(&sel, &picks);
                    if !maximal && stop {
                        break;
                    }
                    let cands: Vec<usize> = (0..sel.options.len()).filter(|j| mask[*j]).collect();
                    let j = if last { cands.last().copied() } else { cands.first().copied() };
                    match j {
                        Some(j) => {
                            picks.push(j);
                            if picks.len() >= sel.max_count {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                if !answers.contains(&picks) {
                    answers.push(picks);
                }
            }
        }
    }
    if answers.is_empty() {
        *stuck = true;
        return false;
    }
    for picks in answers {
        *budget = budget.saturating_sub(1);
        let mut h = Box::new(g.clone());
        let wire = match sel.wire_answer(&picks) {
            Ok(w) => w,
            Err(_) => continue,
        };
        let pr = h.prompts.as_slice()[pi];
        let res = match h.decode_answer(&pr, &wire) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if h.resolve(pi, res).and_then(|_| h.settle()).is_err() {
            continue;
        }
        if explore(&mut h, before, skip, markers, budget, stuck, depth + 1) {
            return true;
        }
    }
    false
}

/// Does the option change the game state under some choices and coin outcome?
fn changes_state(g0: &Game, a: Action, skip: Option<u8>) -> (bool, bool) {
    let markers = skip.is_some();
    let before = sig(g0, skip, markers);
    let mut stuck = false;
    for heads in [false, true] {
        let mut g = Box::new(g0.clone());
        g.rng.force_coins(&[heads; 16]);
        if g.act(a).and_then(|_| g.settle()).is_err() {
            stuck = true;
            continue;
        }
        let mut budget = 60usize;
        if explore(&mut g, &before, skip, markers, &mut budget, &mut stuck, 0) {
            return (true, stuck);
        }
    }
    (false, stuck)
}

fn deck_for(t: DefId) -> Vec<DefId> {
    let td = def(t);
    let n_t = if td.has_tag(ptcg::types::tag::ACE_SPEC) || td.is_trainer() && td.trainer_type == TrainerType::Stadium as u8 { 1 } else { 2 };
    let mut v = vec![];
    for _ in 0..n_t {
        v.push(t);
    }
    let f = def_by_full_name(FILLER).unwrap();
    if td.is_pokemon() {
        for d in stack_for(t) {
            if d != t {
                v.push(d);
            }
        }
    }
    for _ in 0..4 {
        v.push(f);
    }
    // a few extra things for the richer positions
    for n in ["Buneary PRE 83", "Buneary PRE 83", "Noibat PRE 90", "Noibat PRE 90", "Hop's Wooloo JTG 135", "Sacred Charm PFL 93", "Mist Energy TEF 161", "Potion POR 83"] {
        if let Some(d) = def_by_full_name(n) {
            if d != t && !(def(d).has_tag(ptcg::types::tag::ACE_SPEC) && td.has_tag(ptcg::types::tag::ACE_SPEC)) {
                v.push(d);
            }
        }
    }
    let e = def_by_full_name(ENERGY).unwrap();
    while v.len() < 60 {
        v.push(e);
    }
    v
}

fn side(name: &str, kind: char, mine: bool, t: &str, is_pokemon: bool, extra_active: &[&str]) -> Value {
    let mut s = json!({"reset": true});
    let hand_n = match kind { 'A' => 0, 'G' => 7, _ => 3 };
    let mut hand: Vec<String> = vec![];
    if mine {
        if !is_pokemon || true {
            hand.push(t.to_string());
        }
        for _ in 0..hand_n {
            hand.push(fname(ENERGY).to_string());
        }
    }
    if !mine && kind == 'G' {
        hand = vec![fname("Potion POR 83").to_string(), fname(ENERGY).to_string(), fname(ENERGY).to_string(), fname(FILLER).to_string(), fname(FILLER).to_string()];
    }
    if !mine && kind == 'H' {
        hand = vec![fname(ENERGY).to_string(); 3];
    }
    s["hand"] = json!(hand);
    let _ = name;
    s["active"] = json!(if mine && is_pokemon && kind != 'P' { extra_active.iter().map(|x| x.to_string()).collect::<Vec<_>>() } else { vec![fname(FILLER).to_string()] });
    if matches!(kind, 'D' | 'E' | 'F') {
        s["discard"] = json!([fname(FILLER), fname(ENERGY), fname(ENERGY)]);
    }
    if matches!(kind, 'E' | 'F') {
        s["active_damage"] = json!(30);
        s["active_energy"] = json!([fname(ENERGY)]);
        s["bench"] = json!([{"card": fname(FILLER), "energy": [fname(ENERGY)], "damage": 20}]);
        if !mine {
            s["active_tool"] = json!(fname("Sacred Charm PFL 93"));
            s["active_energy"] = json!([fname(ENERGY), fname("Mist Energy TEF 161")]);
            s["active_energy"] = json!([fname(ENERGY), fname("Mist Energy TEF 161")]);
        } else {
            s["active_tool"] = Value::Null;
        }
    }
    if kind == 'G' {
        s["discard"] = json!([fname(FILLER), fname(ENERGY), fname(ENERGY)]);
        if !mine {
            s["bench"] = json!([{"card": fname(FILLER)}, {"card": fname("Buneary PRE 83")}]);
        }
    }
    if kind == 'H' {
        s["discard"] = json!([fname(FILLER), fname(ENERGY)]);
        if mine {
            s["bench"] = json!([{"card": fname("Buneary PRE 83")}, {"card": fname("Noibat PRE 90")}]);
            s["active_conditions"] = json!(["POISONED"]);
        }
    }
    if kind == 'I' {
        s["active_conditions"] = json!(["BURNED", "CONFUSED", "POISONED"]);
    }
    if kind == 'P' {
        s["active_energy"] = json!([fname(ENERGY)]);
        if mine && is_pokemon {
            s["bench"] = json!([{"card": extra_active, "energy": [fname(ENERGY)]}]);
        }
    }
    if kind == 'F' {
        let mut b = vec![];
        for n in ["Buneary PRE 83", "Buneary PRE 83", "Noibat PRE 90", "Noibat PRE 90", "Hop's Wooloo JTG 135"] {
            b.push(json!({"card": fname(n)}));
        }
        s["bench"] = json!(b);
    }
    s
}

fn read_deck(path: &std::path::Path) -> Vec<DefId> {
    let mut v = vec![];
    for l in std::fs::read_to_string(path).unwrap().lines() {
        let l = l.trim();
        if l.is_empty() {
            continue;
        }
        let (n, name) = l.split_once(' ').unwrap();
        let n: usize = n.parse().unwrap_or(1);
        if let Some(d) = def_by_full_name(name) {
            for _ in 0..n {
                v.push(d);
            }
        }
    }
    v
}

/// Random play: at every turn decision, every Trainer / Ability / Stadium option is run and checked for "no change".
fn random_mode(n: usize, seed0: u32, only_pokemon: bool) {
    use ptcg::rng::Rng;
    let mut decks: Vec<Vec<DefId>> = vec![];
    for dir in ["decks/meta", "decks"] {
        let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file() && p.extension().map(|x| x == "txt").unwrap_or(false)).collect();
        paths.sort();
        for p in paths {
            let d = read_deck(&p);
            if d.len() == 60 {
                decks.push(d);
            }
        }
    }
    // trainer pool for salads
    let pool: Value = serde_json::from_str(&std::fs::read_to_string("data/pool.json").unwrap()).unwrap();
    let mut trainers: Vec<DefId> = vec![];
    for c in pool.as_array().unwrap() {
        if let Some(id) = def_by_full_name(c["fullName"].as_str().unwrap()) {
            let d = def(id);
            if d.is_trainer() && !d.has_tag(ptcg::types::tag::ACE_SPEC) && cards_impl(id) {
                trainers.push(id);
            }
        }
    }
    let ability_pokemon: Vec<DefId> = pool
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| def_by_full_name(c["fullName"].as_str().unwrap()))
        .filter(|d| def(*d).is_pokemon() && def(*d).powers.iter().any(|p| p.power_type == PowerType::Ability as u8) && cards_impl(*d))
        .collect();
    let mut stats: std::collections::BTreeMap<String, (u32, u32, String)> = Default::default();
    // ability name -> (uses, offered again for the same Pokemon, offered again for another Pokemon)
    let mut reoffer: std::collections::BTreeMap<String, (u32, u32, u32)> = Default::default();
    let mut rng = Rng::new(seed0 ^ 0x77);
    for i in 0..n {
        let seed = seed0.wrapping_mul(7919).wrapping_add(i as u32);
        let mut d0 = decks[rng.index(decks.len())].clone();
        let mut d1 = decks[rng.index(decks.len())].clone();
        if only_pokemon {
            // an ability Pokemon line (4 of each stage) plus Basic filler and Energy
            for d in [&mut d0, &mut d1] {
                let t = ability_pokemon[rng.index(ability_pokemon.len())];
                let mut v: Vec<DefId> = vec![];
                for x in stack_for(t) {
                    for _ in 0..4 {
                        v.push(x);
                    }
                }
                let f = def_by_full_name(FILLER).unwrap();
                for _ in 0..4 {
                    v.push(f);
                }
                let e = def_by_full_name(ENERGY).unwrap();
                let lt = def_by_full_name("Lightning Energy MEE").unwrap();
                let mut flip = false;
                while v.len() < 60 {
                    v.push(if flip { e } else { lt });
                    flip = !flip;
                }
                v.truncate(60);
                *d = v;
            }
        }
        if !only_pokemon {
        // trainer salad: replace up to 14 random non-Pokemon, non-basic-Energy cards by random trainers
        for d in [&mut d0, &mut d1] {
            let mut cnt: std::collections::HashMap<&str, usize> = Default::default();
            for c in d.iter() {
                *cnt.entry(def(*c).name).or_default() += 1;
            }
            for _ in 0..14 {
                let j = rng.index(d.len());
                let dj = def(d[j]);
                if dj.is_pokemon() || (dj.is_energy() && dj.energy_type == 1) {
                    continue;
                }
                let t = trainers[rng.index(trainers.len())];
                let tn = def(t).name;
                if d.iter().filter(|x| def(**x).name == tn).count() >= 4 {
                    continue;
                }
                d[j] = t;
            }
        }
        }
        let mut g = Box::new(Game::new(seed));
        if g.start([&d0, &d1]).is_err() {
            continue;
        }
        let _ = g.settle();
        let mut prng = Rng::new(seed ^ 0x9e37_79b9);
        for _ in 0..600 {
            if g.st.phase == GamePhase::Finished {
                break;
            }
            match g.pending() {
                Pending::Turn(_) => {
                    let opts = legal_turn_options(&g);
                    if opts.is_empty() {
                        break;
                    }
                    let p = g.st.active_player as usize;
                    for o in &opts {
                        let (label, skip) = match o.action {
                            Action::PlayCard { hand_index, .. } => {
                                let c = g.st.players[p].hand.as_slice()[hand_index as usize];
                                let d = g.st.cdef(c);
                                if !d.is_trainer() {
                                    continue;
                                }
                                (d.full_name.to_string(), Some(c))
                            }
                            Action::UseAbility { name, target } => {
                                let src = ptcg::prompts::get_target(&g.st, p, target).ok().and_then(|t| g.st.slot_pokemon(t.p as usize, t.s));
                                match src {
                                    Some(c) => (format!("{} / {}", g.st.cdef(c).full_name, name), None),
                                    None => continue,
                                }
                            }
                            Action::UseStadium => match g.st.stadium_card() {
                                Some(c) => (format!("{} (stadium use)", g.st.cdef(c).full_name), None),
                                None => continue,
                            },
                            _ => continue,
                        };
                        let before = sig(&g, skip, skip.is_some());
                        let (changed, _) = changes_state(&g, o.action, skip);
                        let e = stats.entry(label).or_insert((0, 0, String::new()));
                        e.0 += 1;
                        if !changed {
                            e.1 += 1;
                            if e.2.is_empty() {
                                let pl = &g.st.players[p];
                                e.2 = format!("seed {} turn {} hand {} deck {} opp_hand {} opp_bench {} before={}", seed, g.st.turn, pl.hand.len(), pl.deck.len(), g.st.players[1 - p].hand.len(), g.st.players[1 - p].bench.iter().filter(|b| !g.st.players[1 - p].slots[**b as usize].cards.is_empty()).count(), &before[..before.len().min(200)]);
                            }
                        }
                    }
                    // play a random option, preferring non-pass
                    let non_pass: Vec<usize> = (0..opts.len()).filter(|k| !matches!(opts[*k].action, Action::Pass)).collect();
                    let abil: Vec<usize> = (0..opts.len()).filter(|k| matches!(opts[*k].action, Action::UseAbility { .. })).collect();
                    let k = if only_pokemon && !abil.is_empty() && prng.below(100) < 60 {
                        abil[prng.index(abil.len())]
                    } else if !non_pass.is_empty() && prng.below(100) < 92 { non_pass[prng.index(non_pass.len())] } else { opts.iter().position(|o| matches!(o.action, Action::Pass)).unwrap_or(0) };
                    let used = if let Action::UseAbility { name, target } = opts[k].action {
                        ptcg::prompts::get_target(&g.st, p, target).ok().and_then(|t| g.st.slot_pokemon(t.p as usize, t.s)).map(|c| (name, c, g.st.cdef(c).full_name))
                    } else {
                        None
                    };
                    if g.act(opts[k].action).and_then(|_| g.settle()).is_err() {
                        break;
                    }
                    if let Some((name, c, cname)) = used {
                        if let (Pending::Turn(_), true) = (g.pending(), g.st.phase == GamePhase::PlayerTurn) {
                            let e = reoffer.entry(format!("{} / {}", cname, name)).or_default();
                            e.0 += 1;
                            for o2 in legal_turn_options(&g) {
                                if let Action::UseAbility { name: n2, target: t2 } = o2.action {
                                    if n2 == name {
                                        let c2 = ptcg::prompts::get_target(&g.st, p, t2).ok().and_then(|t| g.st.slot_pokemon(t.p as usize, t.s));
                                        if c2 == Some(c) {
                                            e.1 += 1;
                                        } else {
                                            e.2 += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Pending::Decision(pi) => {
                    let sel = match g.select() {
                        Ok(Some(sl)) => sl,
                        _ => break,
                    };
                    let nn = sel.options.len();
                    let mut picks: Vec<usize> = vec![];
                    loop {
                        let (mask, stop) = g.pick_mask(&sel, &picks);
                        let allowed: Vec<usize> = (0..nn).filter(|j| mask[*j]).collect();
                        let choices = allowed.len() + stop as usize;
                        if choices == 0 {
                            break;
                        }
                        let r = prng.index(choices);
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
                    let res = match g.decode_answer(&pr, &wire) {
                        Ok(r) => r,
                        Err(_) => break,
                    };
                    if g.resolve(pi, res).and_then(|_| g.settle()).is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    }
    for (k, (u, same, other)) in &reoffer {
        println!("REOFFER {} | used {} again_same {} again_other {}", k, u, same, other);
    }
    for (k, (n, z, ex)) in &stats {
        println!("{} | offered {} noop {} | {}", k, n, z, ex);
    }
}

/// First prompt of every attack of every pool Pokemon (Active with two Energy of each Basic type).
fn attack_prompts() {
    let pool: Value = serde_json::from_str(&std::fs::read_to_string("data/pool.json").unwrap()).unwrap();
    let types = ["Grass Energy MEE", "Fire Energy MEE", "Water Energy MEE", "Lightning Energy MEE", "Psychic Energy MEE", "Fighting Energy MEE", "Darkness Energy MEE", "Metal Energy MEE"];
    for c in pool.as_array().unwrap() {
        let id = match def_by_full_name(c["fullName"].as_str().unwrap()) {
            Some(i) => i,
            None => continue,
        };
        let td = def(id);
        if !td.is_pokemon() || td.attacks.is_empty() || !cards_impl(id) {
            continue;
        }
        let mut deck: Vec<DefId> = vec![];
        for x in stack_for(id) {
            for _ in 0..2 {
                deck.push(x);
            }
        }
        for _ in 0..4 {
            deck.push(def_by_full_name(FILLER).unwrap());
        }
        for t in types {
            for _ in 0..2 {
                deck.push(def_by_full_name(t).unwrap());
            }
        }
        let e = def_by_full_name(ENERGY).unwrap();
        while deck.len() < 60 {
            deck.push(e);
        }
        let mut base = Box::new(Game::new(7));
        base.start([&deck, &deck]).unwrap();
        let _ = base.settle();
        if !reach_turn(&mut base) {
            continue;
        }
        let me = base.st.active_player as usize;
        let stack: Vec<&str> = stack_for(id).iter().map(|d| def(*d).full_name).collect();
        let en: Vec<Value> = types.iter().flat_map(|t| vec![json!(fname(t)), json!(fname(t))]).collect();
        let sc = json!({
            "me": {"reset": true, "active": stack, "active_energy": en, "bench": [{"card": fname(FILLER), "energy": [fname(ENERGY)]}, {"card": fname(FILLER)}], "discard": [fname(FILLER), fname(ENERGY)], "hand": [fname(ENERGY), fname(ENERGY)]},
            "opp": {"reset": true, "active": fname(FILLER), "active_energy": [fname(ENERGY)], "bench": [{"card": fname(FILLER), "energy": [fname(ENERGY)]}, {"card": fname(FILLER)}], "hand": [fname(ENERGY)]}
        });
        let mut g = base.clone();
        if ptcg::scenario::apply(&mut g, &sc).is_err() {
            continue;
        }
        for o in legal_turn_options(&g) {
            if let Action::Attack { name, .. } = o.action {
                let mut h = Box::new(g.clone());
                h.rng.force_coins(&[true; 8]);
                if h.act(o.action).and_then(|_| h.settle()).is_err() {
                    println!("APROMPT {} | {} | error", td.full_name, name);
                    continue;
                }
                match h.pending() {
                    Pending::Decision(pi) => {
                        let msg = h.prompts.as_slice()[pi].message;
                        if let Ok(Some(sel)) = h.select() {
                            // the smallest number of picks (first options) the prompt accepts
                            let n = sel.options.len();
                            let mut acc = String::from("none");
                            for k in 0..=n.min(20) {
                                let picks: Vec<usize> = (0..k).collect();
                                if h.is_accepted_answer(&sel, &picks) {
                                    acc = k.to_string();
                                    break;
                                }
                            }
                            println!("APROMPT {} | {} | {} | min {} max {} of {} | accepts_from {}", td.full_name, name, msg, sel.min_count, sel.max_count, n, acc);
                        }
                    }
                    _ => println!("APROMPT {} | {} | none", td.full_name, name),
                }
            }
        }
    }
}

/// E-05 / E-06 trigger Abilities: what happens right after the Pokemon is played, in positions where the effect does nothing.
fn trigger_probe() {
    let names = [
        "Marnie's Grimmsnarl ex ASC 287", "Noctowl PRE 78", "Hop's Dubwool JTG 136", "Kadabra MEG 55", "Hariyama MEG 73", "Archaludon ex SSP 130", "Alakazam MEG 56",
        "Bloodmoon Ursaluna PRE 54", "Iron Leaves ex PRE 176", "Meowth ex POR 62", "Durant ex SSP 4", "Chien-Pao SSP 56", "Drilbur TEF 85", "Farfetch'd TWM 132",
    ];
    for n in names {
        let t = match def_by_full_name(n) {
            Some(t) => t,
            None => {
                println!("TRIGGER {} | unknown", n);
                continue;
            }
        };
        let td = def(t);
        let mut deck = deck_for(t);
        for x in stack_for(t) {
            if x != t {
                deck.push(x);
            }
        }
        deck.truncate(60);
        let mut base = Box::new(Game::new(7));
        base.start([&deck, &deck]).unwrap();
        let _ = base.settle();
        let mut ok = false;
        for _ in 0..4 {
            if reach_turn(&mut base) && base.st.turn >= 3 {
                ok = true;
                break;
            }
            if base.act(Action::Pass).and_then(|_| base.settle()).is_err() {
                break;
            }
        }
        if !ok {
            println!("TRIGGER {} | no turn 3", td.full_name);
            continue;
        }
        let pre: Vec<&str> = stack_for(t).iter().filter(|d| **d != t).map(|d| def(*d).full_name).collect();
        for kind in ['A', 'C', 'D', 'E'] {
            let mut g = base.clone();
            let me = g.st.active_player as usize;
            let active: Vec<&str> = if pre.is_empty() { vec![fname(FILLER)] } else { pre.clone() };
            let mut sc = json!({"me": side("me", kind, true, td.full_name, false, &active), "opp": side("opp", kind, false, td.full_name, false, &active)});
            sc["me"]["active"] = json!(active);
            if ptcg::scenario::apply(&mut g, &sc).is_err() {
                println!("TRIGGER {} | {} | scenario error", td.full_name, kind);
                continue;
            }
            if kind == 'C' {
                g.move_to(ListRef::Deck(me as u8), ListRef::LostZone(me as u8), None);
            }
            let mut res = String::from("not offered");
            for o in legal_turn_options(&g) {
                if let Action::PlayCard { hand_index, .. } = o.action {
                    let c = g.st.players[me].hand.as_slice()[hand_index as usize];
                    if g.st.cards[c as usize].def != t {
                        continue;
                    }
                    let mut h = Box::new(g.clone());
                    if h.act(o.action).and_then(|_| h.settle()).is_err() {
                        res = "error".into();
                    } else if let Pending::Decision(pi) = h.pending() {
                        let msg = h.prompts.as_slice()[pi].message;
                        res = format!("prompt {}", msg);
                    } else {
                        res = "no prompt".into();
                    }
                    break;
                }
            }
            println!("TRIGGER {} | {} | {}", td.full_name, kind, res);
        }
    }
}

fn cards_impl(id: DefId) -> bool {
    ptcg::cards::impl_for(id).is_some()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--triggers") {
        trigger_probe();
        return;
    }
    if args.iter().any(|a| a == "--attack-prompts") {
        attack_prompts();
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--random") {
        let n: usize = args.get(i + 1).and_then(|x| x.parse().ok()).unwrap_or(200);
        random_mode(n, 1, args.iter().any(|a| a == "--abilities"));
        return;
    }
    let only: Option<Vec<String>> = args.iter().position(|a| a == "--cards").and_then(|i| args.get(i + 1)).map(|s| s.split('|').map(|x| x.to_string()).collect());
    let pool: Value = serde_json::from_str(&std::fs::read_to_string("data/pool.json").unwrap()).unwrap();
    let mut targets: Vec<DefId> = vec![];
    for c in pool.as_array().unwrap() {
        let tw = c["fullName"].as_str().unwrap();
        let id = match def_by_full_name(tw) {
            Some(i) => i,
            None => continue,
        };
        let d = def(id);
        if let Some(o) = &only {
            if !o.iter().any(|n| def_by_full_name(n) == Some(id)) {
                continue;
            }
        } else if !(d.is_trainer() || (d.is_pokemon() && d.powers.iter().any(|p| p.power_type == PowerType::Ability as u8))) {
            continue;
        }
        targets.push(id);
    }
    for t in targets {
        let td = def(t);
        let is_pokemon = td.is_pokemon();
        let decks = deck_for(t);
        let mut base = Box::new(Game::new(7));
        base.start([&decks, &decks]).unwrap();
        let _ = base.settle();
        if !reach_turn(&mut base) {
            println!("{} | setup failed", td.full_name);
            continue;
        }
        let active_stack: Vec<&str> = if is_pokemon { stack_for(t).iter().map(|d| def(*d).full_name).collect() } else { vec![fname(FILLER)] };
        let is_stadium = td.is_trainer() && td.trainer_type == TrainerType::Stadium as u8;
        let kinds: Vec<(char, bool)> = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'P', 'I'].iter().map(|k| (*k, false)).chain(if is_stadium { ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'].iter().map(|k| (*k, true)).collect::<Vec<_>>() } else { vec![] }).collect();
        for (kind, inplay) in kinds {
            let mut g = base.clone();
            let me = g.st.active_player as usize;
            let mut sc = json!({"me": side("me", kind, true, td.full_name, is_pokemon, &active_stack), "opp": side("opp", kind, false, td.full_name, false, &active_stack)});
            if is_pokemon {
                // hand holds only energy
                sc["me"]["hand"] = json!(if kind == 'A' { vec![] } else { vec![fname(ENERGY); 3] });
            }
            if inplay {
                let mut h: Vec<Value> = sc["me"]["hand"].as_array().unwrap().iter().cloned().collect();
                h.retain(|x| x.as_str() != Some(td.full_name));
                sc["me"]["hand"] = json!(h);
                sc["opp"]["stadium"] = json!(td.full_name);
            }
            if let Err(e) = ptcg::scenario::apply(&mut g, &sc) {
                println!("{} | {} | scenario error {}", td.full_name, kind, e);
                continue;
            }
            if kind == 'C' {
                g.move_to(ListRef::Deck(me as u8), ListRef::LostZone(me as u8), None);
            }
            let opts = legal_turn_options(&g);
            let mut offered: Vec<String> = vec![];
            for o in &opts {
                let mut tag: Option<(String, Option<u8>)> = None;
                match o.action {
                    Action::PlayCard { hand_index, .. } if !is_pokemon => {
                        let c = g.st.players[me].hand.as_slice()[hand_index as usize];
                        if g.st.cards[c as usize].def == t && !inplay && !offered.iter().any(|x| x.starts_with("play")) {
                            tag = Some(("play".to_string(), Some(c)));
                        }
                    }
                    Action::UseAbility { name, target } if is_pokemon => tag = Some((format!("{}@{:?}", name, target.slot), None)),
                    Action::UseStadium if !is_pokemon && inplay => tag = Some(("stadium".to_string(), None)),
                    _ => {}
                }
                if let Some((label, skip)) = tag {
                    if args.iter().any(|a| a == "--prompts") {
                        // the first prompt the option opens: message, min, max, number of options
                        let mut h = Box::new(g.clone());
                        h.rng.force_coins(&[true; 8]);
                        if h.act(o.action).and_then(|_| h.settle()).is_ok() {
                            if let Pending::Decision(pi) = h.pending() {
                                let msg = h.prompts.as_slice()[pi].message;
                                if let Ok(Some(sel)) = h.select() {
                                    println!("PROMPT {} | {} | {} | min {} max {} of {}", td.full_name, kind, msg, sel.min_count, sel.max_count, sel.options.len());
                                }
                            }
                        }
                    }
                    let (changed, stuck) = changes_state(&g, o.action, skip);
                    offered.push(format!("{}{}{}", label, if changed { "" } else { ":NOOP" }, if stuck { ":STUCK" } else { "" }));
                }
            }
            println!("{} | {} | {}", td.full_name, if inplay { kind.to_ascii_lowercase() } else { kind }, if offered.is_empty() { "-".to_string() } else { offered.join(",") });
        }
    }
}
