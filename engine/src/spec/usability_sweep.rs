//! User decision D10: the unit sweep over every Trainer play, Ability and Stadium use on synthetic boards. Boards vary what
//! usability reads: empty or small hand / deck / discard pile / Bench, a full Bench, no Energy, damage, the Prize count, the
//! turn, Pokémon Knocked Out last turn, a Supporter played this turn, a Stadium in play, both sides.
//!
//! Before the switch (closeout 4a) it compared the old usability (every `needs` entry and every op's implied precondition)
//! with `run::usable_by_rules`: no difference on 27,814 boards. Since the switch usability is `needs` + the rules' walk, and
//! the sweep keeps `needs` honest: a non-printed entry that fails on boards where the rules alone already say "no" only
//! restates usability and must go (the switch removed 67). `D10_NEEDS_OUT=<file>` writes, per entry, the boards where it
//! fails and where it decides (the rules alone say "yes"), with sample situations; `D10_COVERAGE=1` prints each program's
//! counts. Most cards of a board come from the decks
//! the card is played in (engine/bench/spec.json, decks/), the rest from the whole pool.

use super::run::{usable_by_rules, Frame, Phase, Prog};
use super::*;
use crate::effects::{Effect, PowerRef};
use crate::game::Game;
use crate::list::{CardId, CardList};
use crate::types::{Stage, TrainerType};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// A small deterministic generator (xorshift).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.below(v.len())]
    }
}

struct Pools {
    basics: Vec<u16>,
    pokemon: Vec<u16>,
    energy: Vec<u16>,
    basic_energy: Vec<u16>,
    tools: Vec<u16>,
    stadiums: Vec<u16>,
    any: Vec<u16>,
}

fn pools() -> Pools {
    pools_of((0..crate::carddb::cards().len() as u16).collect(), None)
}

/// The pools of the cards `defs`; a pool left empty takes `fallback`'s.
fn pools_of(defs: Vec<u16>, fallback: Option<&Pools>) -> Pools {
    let mut p = Pools { basics: vec![], pokemon: vec![], energy: vec![], basic_energy: vec![], tools: vec![], stadiums: vec![], any: vec![] };
    for i in defs {
        let d = crate::carddb::def(i);
        if d.full_name.is_empty() {
            continue;
        }
        p.any.push(i);
        if d.is_pokemon() {
            p.pokemon.push(i);
            if d.stage() == Stage::Basic {
                p.basics.push(i);
            }
        } else if d.is_energy() {
            p.energy.push(i);
            if d.energy_type() == crate::types::EnergyType::Basic {
                p.basic_energy.push(i);
            }
        } else if d.is_trainer() {
            match d.trainer_type() {
                TrainerType::Tool => p.tools.push(i),
                TrainerType::Stadium => p.stadiums.push(i),
                _ => {}
            }
        }
    }
    if let Some(fb) = fallback {
        for (mine, theirs) in [(&mut p.basics, &fb.basics), (&mut p.pokemon, &fb.pokemon), (&mut p.energy, &fb.energy), (&mut p.basic_energy, &fb.basic_energy), (&mut p.tools, &fb.tools), (&mut p.stadiums, &fb.stadiums), (&mut p.any, &fb.any)] {
            if mine.is_empty() {
                mine.clone_from(theirs);
            }
        }
    }
    p
}

/// The cards played beside each card in the repository's decks (engine/bench/spec.json, decks/*.txt, decks/meta/*.txt): the
/// boards draw most cards from them, so a card meets what it is played with (Rare Candy and Stage 2 Pokémon, Lunatone and
/// Solrock, Team Rocket's cards together).
fn related() -> std::collections::HashMap<u16, Vec<u16>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut decks: Vec<Vec<u16>> = Vec::new();
    if let Ok(t) = std::fs::read_to_string(root.join("bench/spec.json")) {
        let v: Value = serde_json::from_str(&t).unwrap_or(Value::Null);
        for d in v["decks"].as_array().into_iter().flatten() {
            decks.push(d["cards"].as_array().into_iter().flatten().filter_map(|n| n.as_str().and_then(crate::carddb::def_by_full_name)).collect());
        }
    }
    for dir in [root.join("../decks"), root.join("../decks/meta")] {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let Ok(t) = std::fs::read_to_string(e.path()) else { continue };
            let deck: Vec<u16> = t.lines().filter_map(|l| l.trim().split_once(' ').and_then(|(_, n)| crate::carddb::def_by_full_name(n.trim()))).collect();
            if !deck.is_empty() {
                decks.push(deck);
            }
        }
    }
    let mut m: std::collections::HashMap<u16, Vec<u16>> = std::collections::HashMap::new();
    for deck in &decks {
        for &c in deck {
            let e = m.entry(c).or_default();
            for &o in deck {
                if !e.contains(&o) {
                    e.push(o);
                }
            }
        }
    }
    m
}

/// What is being checked.
#[derive(Clone, Copy)]
enum Subject {
    Play,
    Power(u8),
    UseStadium,
}

fn name(d: u16) -> &'static str {
    crate::carddb::def(d).full_name
}

/// A card the board names: its deck holds it.
fn need(d: u16, deck: &mut Vec<u16>) -> &'static str {
    deck.push(d);
    name(d)
}

/// One side of a synthetic board: the scenario JSON and the cards its deck must hold.
fn side(r: &mut Rng, pl: &Pools, active: u16, on_bench: Option<u16>, in_hand: &[u16], deck: &mut Vec<u16>) -> (Value, usize) {
    let mut s = json!({"reset": true});
    s["active"] = json!(need(active, deck));
    let energy_n = [0, 0, 1, 2, 3][r.below(5)];
    let en: Vec<&str> = (0..energy_n).map(|_| { let e = if r.below(3) == 0 { *r.pick(&pl.energy) } else { *r.pick(&pl.basic_energy) }; need(e, deck) }).collect();
    s["active_energy"] = json!(en);
    if r.below(4) == 0 {
        s["active_damage"] = json!(20);
    }
    if r.below(5) == 0 {
        s["active_tool"] = json!(need(*r.pick(&pl.tools), deck));
    }
    let bench_n = [0, 0, 1, 2, 3, 5][r.below(6)];
    let mut bench = Vec::new();
    if let Some(b) = on_bench {
        bench.push(json!({"card": need(b, deck)}));
    }
    while bench.len() < bench_n.max(on_bench.is_some() as usize) {
        let b = if r.below(4) == 0 { *r.pick(&pl.pokemon) } else { *r.pick(&pl.basics) };
        let mut e = json!({"card": need(b, deck)});
        if r.below(2) == 0 {
            e["energy"] = json!([need(*r.pick(&pl.basic_energy), deck)]);
        }
        if r.below(4) == 0 {
            e["damage"] = json!(10);
        }
        bench.push(e);
    }
    s["bench"] = json!(bench);
    let hand_n = [0, 0, 1, 2, 3, 6][r.below(6)];
    let mut hand: Vec<&str> = in_hand.iter().map(|d| need(*d, deck)).collect();
    for _ in 0..hand_n {
        hand.push(need(*r.pick(&pl.any), deck));
    }
    s["hand"] = json!(hand);
    let disc_n = [0, 0, 1, 3, 10][r.below(5)];
    let disc: Vec<&str> = (0..disc_n)
        .map(|_| {
            let d = match r.below(3) {
                0 => *r.pick(&pl.basic_energy),
                1 => *r.pick(&pl.pokemon),
                _ => *r.pick(&pl.any),
            };
            need(d, deck)
        })
        .collect();
    s["discard"] = json!(disc);
    s["prizes_left"] = json!(1 + r.below(6));
    if r.below(4) == 0 {
        s["supporter_played"] = json!(true);
    }
    let deck_left = [0, 1, 2, 5, 20][r.below(5)];
    (s, deck_left)
}

/// The usability of `subject` of card `me` for player `p` and the non-printed `needs` entries that fail, `usize::MAX` last
/// when the rules alone would say "yes" (`None`: a printed condition fails, or a check errs).
fn answers(g: &mut Game, me: CardId, p: usize, subject: Subject) -> Option<(bool, Vec<usize>)> {
    let spec = crate::cards::spec_for(g.st.cards[me as usize].def)?;
    let (prog, needs, steps, e) = match subject {
        Subject::Play => {
            let play = spec.play.as_ref()?;
            if g.st.cdef(me).trainer_type() == TrainerType::Item {
                crate::engine::play_trainer::enter_item_play(g, p, me);
                if let Some(w) = crate::engine::play_trainer::companion(g, p, me) {
                    crate::engine::play_trainer::enter_item_play(g, p, w);
                }
            }
            let cause = crate::cause::Cause::rule(crate::cause::RuleWhich::Action, p as u8);
            let e = g.new_fx(Effect::PlayTrainer { p: p as u8, card: me, target: None, use_: crate::spec::event::TrainerUse::Played, cause, with: None });
            (Prog::Play, play.needs, play.steps, e)
        }
        Subject::Power(i) => {
            let pw = &spec.powers[i as usize];
            let e = g.new_fx(Effect::Power { p: p as u8, power: PowerRef { card: me, index: pw.index }, card: me, target: None });
            (Prog::Power(i), pw.needs, pw.steps, e)
        }
        Subject::UseStadium => {
            let us = spec.use_stadium.as_ref()?;
            let e = g.new_fx(Effect::UseStadium { p: p as u8, stadium: me });
            (Prog::UseStadium, us.needs, us.steps, e)
        }
    };
    let f = Frame::start(g, me, prog, Phase::Use, e, p, false);
    let mut failing = Vec::new();
    for (i, c) in needs.iter().enumerate() {
        if !matches!(crate::spec::value::cond_m(g, me, &f, c), Ok(true)) {
            if matches!(c, Cond::Printed(_)) {
                return None;
            }
            failing.push(i);
        }
    }
    let new = failing.is_empty() && usable_by_rules(g, me, &f, steps);
    // The rules' answer with the printed conditions only: where it differs from `new`, a non-printed `needs` entry decides.
    let bare = usable_by_rules(g, me, &f, steps);
    if bare != new {
        failing.push(usize::MAX);
    }
    Some((new, failing))
}

fn situation(g: &Game, p: usize, me: CardId) -> String {
    let pl = &g.st.players[p];
    let o = &g.st.players[1 - p];
    let bench = |q: &crate::state::Player| q.bench.iter().filter(|b| !q.slots[**b as usize].cards.is_empty()).count();
    let energy = |q: &crate::state::Player| q.slots[q.active as usize].energies.len();
    format!(
        "turn {} hand {} deck {} discard {} bench {} active energy {} | opp hand {} deck {} discard {} bench {} active energy {}{}",
        g.st.turn,
        pl.hand.iter().filter(|c| *c != me).count(),
        pl.deck.len(),
        pl.discard.len(),
        bench(pl),
        energy(pl),
        o.hand.len(),
        o.deck.len(),
        o.discard.len(),
        bench(o),
        energy(o),
        if (0..2).any(|q| !g.lst(crate::state::ListRef::Stadium(q)).is_empty()) { " | a Stadium in play" } else { "" }
    )
}

/// Build the board for `card` under test and return the game, the turn player and the card.
fn board(r: &mut Rng, all: &Pools, rel: &Pools, card: u16, subject: Subject) -> Option<(Game, usize, CardId)> {
    let pl = if r.below(3) == 0 { all } else { rel };
    let d = crate::carddb::def(card);
    let mut me_deck: Vec<u16> = Vec::new();
    let mut opp_deck: Vec<u16> = Vec::new();
    let (active, on_bench, in_hand): (u16, Option<u16>, Vec<u16>) = match subject {
        Subject::Power(_) if r.below(2) == 0 => (card, None, vec![]),
        Subject::Power(_) => (*r.pick(&pl.basics), Some(card), vec![]),
        Subject::Play => {
            let together = crate::cards::spec_for(card).map_or(false, |s| s.together.is_some());
            (*r.pick(&pl.basics), None, if together { vec![card, card] } else { vec![card] })
        }
        Subject::UseStadium => (*r.pick(&pl.basics), None, vec![]),
    };
    let (mut me_sc, me_left) = side(r, pl, active, on_bench, &in_hand, &mut me_deck);
    let opp_active = *r.pick(&pl.basics);
    let (opp_sc, opp_left) = side(r, pl, opp_active, None, &[], &mut opp_deck);
    let stadium = match subject {
        Subject::UseStadium => Some(card),
        _ if r.below(4) == 0 => Some(*r.pick(&pl.stadiums)),
        _ => None,
    };
    let stadium_mine = r.below(2) == 0;
    if let Some(s) = stadium {
        if stadium_mine || matches!(subject, Subject::UseStadium) {
            me_deck.push(s);
            me_sc["stadium"] = json!(name(s));
        } else {
            opp_deck.push(s);
            let mut o = opp_sc.clone();
            o["stadium"] = json!(name(s));
            return finish(r, pl, card, d, subject, me_deck, opp_deck, me_sc, o, me_left, opp_left);
        }
    }
    finish(r, pl, card, d, subject, me_deck, opp_deck, me_sc, opp_sc, me_left, opp_left)
}

#[allow(clippy::too_many_arguments)]
fn finish(r: &mut Rng, pl: &Pools, card: u16, d: &crate::carddb::CardDef, subject: Subject, mut me_deck: Vec<u16>, mut opp_deck: Vec<u16>, me_sc: Value, opp_sc: Value, me_left: usize, opp_left: usize) -> Option<(Game, usize, CardId)> {
    let _ = d;
    for deck in [&mut me_deck, &mut opp_deck] {
        if deck.len() > 60 {
            return None;
        }
        while deck.len() < 60 {
            deck.push(*r.pick(&pl.basic_energy));
        }
    }
    let mut g = Game::new(1 + r.below(1 << 20) as u32);
    if !crate::engine::setup::deck_is_valid(&g.st, &me_deck) || !crate::engine::setup::deck_is_valid(&g.st, &opp_deck) {
        return None;
    }
    g.start([&me_deck, &opp_deck]).ok()?;
    g.settle().ok();
    if g.st.winner != crate::types::WINNER_NONE {
        return None;
    }
    // Deck 0 is the side under test: it must be the turn player.
    if g.st.active_player != 0 {
        return None;
    }
    let sc = json!({"me": me_sc, "opp": opp_sc});
    crate::scenario::apply(&mut g, &sc).ok()?;
    let p = g.st.active_player as usize;
    for (q, left) in [(p, me_left), (1 - p, opp_left)] {
        let n = g.st.players[q].deck.len().saturating_sub(left);
        if n > 0 {
            g.move_to(crate::state::ListRef::Deck(q as u8), crate::state::ListRef::LostZone(q as u8), Some(n));
        }
    }
    g.st.turn = [1, 2, 2, 3, 5][r.below(5)] + if p == 1 { 1 } else { 0 };
    // What happened before: Pokémon Knocked Out during the opponent's last turn, an Ancient Pokémon's attack, a Supporter
    // played this turn.
    for q in [p, 1 - p] {
        if r.below(3) == 0 {
            let d = *r.pick(&pl.pokemon);
            g.st.players[q].pokemon_knocked_out_last_turn_entries.push(d);
            g.st.players[q].pokemon_knocked_out_last_turn_by_attack.push(r.below(2) == 0);
        }
        // The player's last attack (a Pokémon of theirs, during their previous turn): drawn where the retired flag was.
        let attacked = r.below(4) == 0;
        let mon = (0..g.st.n_cards).find(|&c| g.st.owner(c) == q && g.st.cdef(c).is_pokemon() && c != 255);
        if let (true, Some(c)) = (attacked, mon) {
            g.st.player_last_attack[q] = Some((crate::state::AttackRef { card: c, index: 0 }, c));
            g.st.player_last_attack_turn[q] = g.st.turn - 2;
        }
    }
    if r.below(4) == 0 {
        let sup: Vec<CardId> = (0..g.st.n_cards).filter(|&c| g.st.owner(c) == p && g.st.cdef(c).is_trainer() && g.st.cdef(c).trainer_type() == TrainerType::Supporter && c != 255).collect();
        if !sup.is_empty() {
            let c = *r.pick(&sup);
            g.st.players[p].played_this_turn.push(c);
        }
    }
    crate::spec::passive::lock_sync(&mut g);
    let pl = &g.st.players[p];
    let me = match subject {
        Subject::Play => pl.hand.iter().find(|c| g.st.cards[*c as usize].def == card)?,
        Subject::Power(_) => {
            let mut slots = vec![pl.active];
            slots.extend(pl.bench.iter().copied());
            slots.iter().flat_map(|s| pl.slots[*s as usize].cards.iter().collect::<Vec<_>>()).find(|c| g.st.cards[*c as usize].def == card)?
        }
        Subject::UseStadium => (0..2).flat_map(|q| g.lst(crate::state::ListRef::Stadium(q)).iter().copied().collect::<Vec<CardId>>()).find(|c| g.st.cards[*c as usize].def == card)?,
    };
    Some((g, p, me))
}

/// Every program's differences on `boards` synthetic boards each: (card, program, today, rules) -> (count, situations).
pub(crate) fn sweep(boards: usize) -> (BTreeMap<(String, String, usize), (usize, usize)>, [usize; 4]) {
    // Programs, boards built, boards whose needs held (answered), answered boards where an answer is "yes".
    let mut stats = [0usize; 4];
    let all = pools();
    let rel_map = related();
    let mut needs_out: BTreeMap<(String, String, String), (usize, Vec<String>)> = BTreeMap::new();
    // Per non-printed `needs` entry: boards where it fails, and where it fails while the rules alone say yes.
    let mut entries: BTreeMap<(String, String, usize), (usize, usize)> = BTreeMap::new();
    let mut seen = std::collections::HashSet::new();
    for (i, d) in crate::carddb::cards().iter().enumerate() {
        let card = i as u16;
        let Some(spec) = crate::cards::spec_for(card) else { continue };
        // One def per spec and program (reprints share one).
        if !seen.insert((spec as *const CardSpec as usize, d.name)) {
            continue;
        }
        let mut subjects = Vec::new();
        if spec.play.is_some() && d.is_trainer() {
            subjects.push((Subject::Play, "play".to_string()));
        }
        for (k, pw) in spec.powers.iter().enumerate() {
            let pd = d.powers.get(pw.index as usize);
            if pd.map_or(false, |x| x.use_when_in_play) {
                subjects.push((Subject::Power(k as u8), format!("Ability {}", pd.map_or("", |x| x.name))));
            }
        }
        if spec.use_stadium.is_some() {
            subjects.push((Subject::UseStadium, "use".to_string()));
        }
        let rel = pools_of(rel_map.get(&card).cloned().unwrap_or_default(), Some(&all));
        for (subject, label) in subjects {
            stats[0] += 1;
            let mut cov = [0usize; 3];
            let mut r = Rng(0x9E37_79B9_7F4A_7C15 ^ ((card as u64) << 20) ^ label.len() as u64);
            for _ in 0..boards {
                let Some((mut g, p, me)) = board(&mut r, &all, &rel, card, subject) else { continue };
                stats[1] += 1;
                let sit = situation(&g, p, me);
                let Some((new, failing)) = answers(&mut g, me, p, subject) else { continue };
                let decisive = failing.last() == Some(&usize::MAX);
                for &i in failing.iter().filter(|i| **i != usize::MAX) {
                    let e = entries.entry((d.full_name.to_string(), label.clone(), i)).or_insert((0, 0));
                    e.0 += 1;
                    e.1 += decisive as usize;
                }
                if failing.last() == Some(&usize::MAX) {
                    let idx: Vec<String> = failing.iter().filter(|i| **i != usize::MAX).map(|i| i.to_string()).collect();
                    let e = needs_out.entry((d.full_name.to_string(), label.clone(), idx.join(","))).or_insert((0, Vec::new()));
                    e.0 += 1;
                    if e.1.len() < 2 {
                        e.1.push(sit.clone());
                    }
                }
                stats[2] += 1;
                stats[3] += new as usize;
                cov[0] += 1;
                cov[1] += failing.is_empty() as usize;
                cov[2] += new as usize;
            }
            if std::env::var("D10_COVERAGE").is_ok() {
                eprintln!("COVERAGE {} ({}): printed conditions held {} / {}, every needs entry held {}, usable {}", d.full_name, label, cov[0], boards, cov[1], cov[2]);
            }
        }
    }
    if let Ok(path) = std::env::var("D10_NEEDS_OUT") {
        let mut s = String::new();
        for ((card, prog, idx), (n, sits)) in &needs_out {
            s.push_str(&format!("{} ({}): needs entries [{}] say no where the rules alone say yes; {} boards\n", card, prog, idx, n));
            for x in sits {
                s.push_str(&format!("    {}\n", x));
            }
        }
        s.push_str("\nper entry: card (program) [index]: fails, decisive\n");
        for ((card, prog, i), (n, k)) in &entries {
            s.push_str(&format!("ENTRY {} ({}) [{}]: fails {}, decisive {}\n", card, prog, i, n, k));
        }
        std::fs::write(path, s).unwrap();
    }
    (entries, stats)
}

#[test]
fn d10_no_needs_entry_restates_usability() {
    let (entries, stats) = sweep(160);
    let head = format!("programs {}, boards built {}, printed conditions held {}, usable {}", stats[0], stats[1], stats[2], stats[3]);
    // The boards must reach the programs: most build, and most pass the printed conditions.
    assert!(stats[1] * 2 > stats[0] * 160 && stats[2] * 2 > stats[1], "sweep coverage collapsed: {}", head);
    // An entry that fails on 10 boards or more and never decides restates what the rules' walk derives.
    let restating: Vec<String> = entries.iter().filter(|(_, (n, k))| *n >= 10 && *k == 0).map(|((c, p, i), (n, _))| format!("{} ({}) needs[{}]: fails on {} boards, never decides", c, p, i, n)).collect();
    assert!(restating.is_empty(), "{}
needs entries that only restate usability (remove them):
{}", head, restating.join("
"));
}
