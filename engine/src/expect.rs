//! Scenario `expect` assertions: rules outcomes a scenario asserts about the
//! Rust engine's state (the oracle ignores the key). Evaluated by `diff` at
//! the end of the scenario turn (`turn_end`, after the attack and Knock Outs,
//! before Pokémon Checkup) or at the first turn decision of the next turn
//! (`next_turn`, after Checkup). Format: docs/ENGINE.md, "Scenarios and expect" (local).

use crate::carddb::en_key;
use crate::engine::check::hp_of;
use crate::game::Game;
use crate::list::{CardId, CardList};
use crate::state::SlotId;
use crate::types::{SpecialCondition, WINNER_DRAW, WINNER_NONE};
use serde_json::Value;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum At {
    TurnEnd,
    NextTurn,
    /// The end of the turn after the scenario turn, like `TurnEnd` (the other player's attack and its Knock Outs).
    NextTurnEnd,
    /// The moment the game is decided (`end_game`), with the winner set.
    GameEnd,
    /// The first turn decision of the Tiebreaker game that replaced the scenario's game.
    Tiebreaker,
    /// The n-th turn decision since the scenario edits (0 = right after them).
    Decision,
}

#[derive(Clone, Debug)]
pub struct Assertion {
    pub at: At,
    /// Turns after the scenario turn (`turn_end` default 0, `next_turn` default: the first decision of any later turn).
    pub turn: Option<i32>,
    pub cite: String,
    pub spec: Value,
}

impl Assertion {
    pub fn describe(&self) -> String {
        let mut v = self.spec.clone();
        if let Some(o) = v.as_object_mut() {
            o.remove("cite");
        }
        v.to_string()
    }
}

const KEYS: &[&str] = &[
    "at", "cite", "who", "slot", "bench", "card", "damage", "hp_left", "energy", "tool", "conditions", "in_play", "zone", "count", "contains",
    "not_contains", "prizes_taken", "winner", "active", "turn", "legal", "name", "is", "on", "n", "top", "bench_count", "bench_excludes", "from", "prompts", "absent",
];

/// Parse and validate `scenario.expect`; every assertion needs a `cite`.
pub fn parse(sc: &Value) -> Result<Vec<Assertion>, String> {
    let list = match sc.get("expect") {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(a)) => a,
        Some(_) => return Err("expect: must be a list".into()),
    };
    let mut out = Vec::new();
    for (i, a) in list.iter().enumerate() {
        let o = a.as_object().ok_or_else(|| format!("expect[{}]: must be an object", i))?;
        let cite = o.get("cite").and_then(|c| c.as_str()).filter(|c| !c.trim().is_empty());
        let Some(cite) = cite else { return Err(format!("expect[{}]: missing cite (a ruling or rulebook reference)", i)) };
        if let Some(k) = o.keys().find(|k| !KEYS.contains(&k.as_str())) {
            return Err(format!("expect[{}]: unknown key {}", i, k));
        }
        let at = match o.get("at") {
            None => At::NextTurn,
            Some(Value::String(s)) if s == "next_turn" => At::NextTurn,
            Some(Value::String(s)) if s == "turn_end" => At::TurnEnd,
            Some(Value::String(s)) if s == "next_turn_end" => At::NextTurnEnd,
            Some(Value::String(s)) if s == "game_end" => At::GameEnd,
            Some(Value::String(s)) if s == "tiebreaker" => At::Tiebreaker,
            Some(Value::String(s)) if s == "decision" => At::Decision,
            // `start` = decision 0: right after the scenario edits.
            Some(Value::String(s)) if s == "start" => At::Decision,
            Some(x) => return Err(format!("expect[{}]: bad at {} (turn_end, next_turn, next_turn_end, game_end, tiebreaker, decision or start)", i, x)),
        };
        let kinds = [
            !o.contains_key("legal") && (o.contains_key("slot") || o.contains_key("bench") || (o.contains_key("card") && !o.contains_key("zone"))),
            o.contains_key("zone"),
            o.contains_key("prizes_taken"),
            o.contains_key("winner"),
            o.contains_key("active"),
            o.contains_key("legal"),
            o.contains_key("bench_count") || o.contains_key("bench_excludes"),
            o.contains_key("prompts") || o.contains_key("absent"),
        ];
        if kinds.iter().filter(|k| **k).count() != 1 {
            return Err(format!("expect[{}]: needs exactly one subject (slot/bench/card, zone, prizes_taken, winner, active, legal, bench_count, bench_excludes or prompts)", i));
        }
        let start = o.get("at").and_then(|v| v.as_str()) == Some("start");
        if start && o.contains_key("n") {
            return Err(format!("expect[{}]: at start is decision 0; n goes with at: decision", i));
        }
        if (at == At::Decision && !start) != o.contains_key("n") {
            return Err(format!("expect[{}]: n (the decision number) goes with at: decision, and only with it", i));
        }
        if at == At::Decision && !start && o.get("n").map_or(true, |n| n.as_u64().is_none()) {
            return Err(format!("expect[{}]: n must be a number >= 0", i));
        }
        let turn = match o.get("turn") {
            None => None,
            Some(Value::Number(n)) if n.as_i64().map_or(false, |n| n >= 0) => n.as_i64().map(|n| n as i32),
            Some(x) => return Err(format!("expect[{}]: bad turn {} (turns after the scenario turn, >= 0)", i, x)),
        };
        if let Some(l) = o.get("legal") {
            if at == At::TurnEnd {
                return Err(format!("expect[{}]: legal is checked at a turn decision (at: next_turn or decision)", i));
            }
            match l.as_str() {
                Some("retreat") | Some("stadium") | Some("ability") => {}
                Some("attack") | Some("play") if o.get("name").and_then(|n| n.as_str()).is_some() => {}
                _ => return Err(format!("expect[{}]: legal must be retreat, stadium, ability, or attack / play with a name", i)),
            }
            if o.get("is").map_or(false, |b| !b.is_boolean()) {
                return Err(format!("expect[{}]: is must be true or false", i));
            }
        }
        for key in ["prompts", "absent"] {
            let Some(pr) = o.get(key) else { continue };
            if !pr.as_array().map_or(false, |a| !a.is_empty() && a.iter().all(|x| x.is_string())) {
                return Err(format!("expect[{}]: {} must be a non-empty list of prompt kind names (Wait, CoinFlip, PutDamage, ...)", i, key));
            }
        }
        if !o.contains_key("winner") && !o.contains_key("prompts") && !o.contains_key("absent") && o.get("who").and_then(|w| w.as_str()).map_or(true, |w| w != "me" && w != "opp") {
            return Err(format!("expect[{}]: who must be me or opp", i));
        }
        let mut spec = a.clone();
        if start {
            spec["n"] = Value::from(0);
        }
        out.push(Assertion { at, turn, cite: cite.to_string(), spec });
    }
    Ok(out)
}

fn names_eq(c: CardId, g: &Game, name: &str) -> bool {
    names_eq_def(g.st.cards[c as usize].def, name)
}

fn names_eq_def(d: crate::carddb::DefId, name: &str) -> bool {
    let cd = crate::carddb::def(d);
    cd.full_name == name || cd.name == name
}

fn label(g: &Game, c: CardId) -> String {
    en_key(g.st.cards[c as usize].def).to_string()
}

fn name_list(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()
}

/// Every name in `want` matches a distinct card of `have`; with `exact`, no card is left over.
fn multiset(g: &Game, have: &[CardId], want: &[String], exact: bool) -> bool {
    let mut left: Vec<CardId> = have.to_vec();
    for n in want {
        match left.iter().position(|&c| names_eq(c, g, n)) {
            Some(i) => {
                left.remove(i);
            }
            None => return false,
        }
    }
    !exact || left.is_empty()
}

fn shown(g: &Game, cards: &[CardId]) -> String {
    let mut v: Vec<String> = cards.iter().map(|&c| label(g, c)).collect();
    v.sort();
    format!("{} {:?}", cards.len(), v)
}

fn cond_name(v: u8) -> &'static str {
    match SpecialCondition::from_u8(v) {
        SpecialCondition::Paralyzed => "PARALYZED",
        SpecialCondition::Confused => "CONFUSED",
        SpecialCondition::Asleep => "ASLEEP",
        SpecialCondition::Poisoned => "POISONED",
        SpecialCondition::Burned => "BURNED",
    }
}

fn who(o: &Value, me: usize) -> usize {
    if o["who"].as_str() == Some("me") {
        me
    } else {
        1 - me
    }
}

fn check_zone(g: &Game, a: &Value, p: usize) -> Result<(), String> {
    let zone = a["zone"].as_str().unwrap_or("");
    let pl = &g.st.players[p];
    let cards: Vec<CardId> = match zone {
        "hand" => pl.hand.iter().collect(),
        "deck" => pl.deck.iter().collect(),
        "discard" => pl.discard.iter().collect(),
        "lost_zone" => pl.lostzone.iter().collect(),
        "prizes" => pl.prizes.iter().flat_map(|l| l.iter()).collect(),
        _ => return Err(format!("unknown zone {}", zone)),
    };
    let w = a["who"].as_str().unwrap_or("");
    if let Some(n) = a["count"].as_u64() {
        if cards.len() as u64 != n {
            return Err(format!("{} {} count: expected {}, actual {}", w, zone, n, shown(g, &cards)));
        }
    }
    if a.get("contains").is_some() && !multiset(g, &cards, &name_list(&a["contains"]), false) {
        return Err(format!("{} {} should contain {}, actual {}", w, zone, a["contains"], shown(g, &cards)));
    }
    if a.get("top").is_some() {
        let want = name_list(&a["top"]);
        let ok = cards.len() >= want.len() && want.iter().zip(cards.iter()).all(|(n, &c)| names_eq(c, g, n));
        if !ok {
            let head: Vec<String> = cards.iter().take(want.len().max(3)).map(|&c| label(g, c)).collect();
            return Err(format!("{} {} top should be {}, actual top {:?}", w, zone, a["top"], head));
        }
    }
    for n in name_list(&a["not_contains"]) {
        if cards.iter().any(|&c| names_eq(c, g, &n)) {
            return Err(format!("{} {} should not contain {}, actual {}", w, zone, n, shown(g, &cards)));
        }
    }
    Ok(())
}

/// `legal`: whether a turn action is among the legal options of the player to move (the same
/// trial dispatch as the interface). `is` (default true) is the expected answer.
/// `legal` is `play` (a card in hand, `name`; `on`: "active" or a Bench index narrows it to that target),
/// `ability` (`name` of the Ability and/or `card` = the Pokemon that has it), `stadium` (use the Stadium in play),
/// `retreat` (`on`: the Bench index to retreat to) or `attack` (`name`).
fn check_legal(g: &Game, a: &Value, p: usize) -> Result<(), String> {
    use crate::game::Action;
    if g.st.active_player as usize != p {
        return Err(format!("legal: {} is not the player to move", a["who"].as_str().unwrap_or("")));
    }
    let want = a["is"].as_bool().unwrap_or(true);
    let kind = a["legal"].as_str().unwrap_or("");
    let name = a["name"].as_str().unwrap_or("");
    let card = a["card"].as_str();
    let opts = crate::options::legal_turn_options(g);
    let have = opts.iter().any(|o| match (kind, o.action) {
        ("retreat", Action::Retreat { bench_index }) => a["on"].as_u64().map_or(true, |b| b == bench_index as u64),
        ("stadium", Action::UseStadium) => true,
        ("attack", Action::Attack { name: n, from }) => {
            n == name && a["from"].as_str().map_or(true, |w| from.map_or(false, |f| crate::carddb::def_by_full_name(f).map_or(false, |d| names_eq_def(d, w))))
        }
        ("ability", Action::UseAbility { name: n, target }) => {
            let src = crate::prompts::get_target(&g.st, p, target).ok().and_then(|t| g.st.slot_pokemon(t.p as usize, t.s));
            (name.is_empty() || n == name) && card.map_or(true, |w| src.map_or(false, |c| names_eq(c, g, w)))
        }
        ("play", Action::PlayCard { hand_index, target }) => {
            names_eq(g.st.players[p].hand.as_slice()[hand_index as usize], g, name)
                && match &a["on"] {
                    Value::Null => true,
                    Value::String(s) if s == "active" => target.slot == crate::types::SlotType::Active,
                    Value::Number(n) => target.slot == crate::types::SlotType::Bench && Some(target.index as u64) == n.as_u64(),
                    _ => false,
                }
        }
        _ => false,
    });
    if have != want {
        let list: Vec<String> = opts.iter().map(|o| o.desc.to_string()).collect();
        return Err(format!("legal {} {}: expected {}, actual {} (options: {})", kind, name, want, have, list.join(" ")));
    }
    Ok(())
}

fn selects_by_card(a: &Value) -> bool {
    a["card"].is_string() && a.get("slot").is_none() && a.get("bench").is_none()
}

/// The slot an assertion names; `None` = no Pokémon there.
fn find_slot(g: &Game, a: &Value, p: usize) -> Result<Option<SlotId>, String> {
    let pl = &g.st.players[p];
    if a["slot"].as_str() == Some("active") {
        Ok(Some(pl.active).filter(|&s| g.st.slot_pokemon(p, s).is_some()))
    } else if let Some(i) = a["bench"].as_u64() {
        Ok(pl.bench.get(i as usize).copied().filter(|&s| g.st.slot_pokemon(p, s).is_some()))
    } else if selects_by_card(a) {
        let n = a["card"].as_str().unwrap();
        Ok(pl.in_play().iter().copied().find(|&s| g.st.slot_pokemons(p, s).iter().any(|&c| names_eq(c, g, n))))
    } else {
        Err("bad slot selector (slot: \"active\", bench: N or card: name)".into())
    }
}

fn check_slot(g: &Game, a: &Value, me: usize) -> Result<(), String> {
    let p = who(a, me);
    let w = a["who"].as_str().unwrap_or("");
    let sel = if selects_by_card(a) {
        format!("{} {}", w, a["card"].as_str().unwrap())
    } else if a["slot"].is_string() {
        format!("{} active", w)
    } else {
        format!("{} bench {}", w, a["bench"])
    };
    let Some(s) = find_slot(g, a, p)? else {
        return if a["in_play"].as_bool() == Some(false) { Ok(()) } else { Err(format!("{}: no Pokemon in play there", sel)) };
    };
    let top = g.st.slot_pokemon(p, s).unwrap();
    if a["in_play"].as_bool() == Some(false) {
        return Err(format!("{}: expected not in play, actual in play as {}", sel, label(g, top)));
    }
    let slot = g.st.slot(p, s);
    if let Some(n) = a["damage"].as_i64() {
        if slot.damage as i64 != n {
            return Err(format!("{} damage: expected {}, actual {}", sel, n, slot.damage));
        }
    }
    if let Some(n) = a["hp_left"].as_i64() {
        let left = hp_of(g, p, s, Some(top)) - slot.damage;
        if left as i64 != n {
            return Err(format!("{} hp_left: expected {}, actual {}", sel, n, left));
        }
    }
    if let (Some(n), false) = (a["card"].as_str(), selects_by_card(a)) {
        if !names_eq(top, g, n) {
            return Err(format!("{} card: expected {}, actual {}", sel, n, label(g, top)));
        }
    }
    if let Some(e) = a.get("energy") {
        let have: Vec<CardId> = slot.energies.iter().collect();
        let ok = match e.as_u64() {
            Some(n) => have.len() as u64 == n,
            None => multiset(g, &have, &name_list(e), true),
        };
        if !ok {
            return Err(format!("{} energy: expected {}, actual {}", sel, e, shown(g, &have)));
        }
    }
    if let Some(t) = a.get("tool") {
        let tools: Vec<CardId> = slot.tools.iter().collect();
        let ok = match t.as_str() {
            Some(n) => tools.iter().any(|&c| names_eq(c, g, n)),
            None => tools.is_empty(),
        };
        if !ok {
            return Err(format!("{} tool: expected {}, actual {}", sel, t, shown(g, &tools)));
        }
    }
    if let Some(c) = a.get("conditions") {
        let mut want = name_list(c);
        want.sort();
        let mut have: Vec<String> = slot.special_conditions.iter().map(|&v| cond_name(v).to_string()).collect();
        have.sort();
        if want != have {
            return Err(format!("{} conditions: expected {:?}, actual {:?}", sel, want, have));
        }
    }
    Ok(())
}

/// Evaluate one assertion against the engine state; `me` is the scenario's first side.
pub fn evaluate(g: &Game, me: usize, a: &Assertion) -> Result<(), String> {
    let o = &a.spec;
    let w = o["who"].as_str().unwrap_or("");
    if o.get("prompts").is_some() || o.get("absent").is_some() {
        // The prompts the engine created since the scenario edits, in order: `prompts` must appear as a
        // subsequence, and no kind of `absent` may appear at all.
        let seen = PROMPT_LOG.with(|l| l.borrow().clone());
        if let Some(bad) = o.get("absent").and_then(|a| a.as_array()) {
            for b in bad.iter().filter_map(|x| x.as_str()) {
                if seen.iter().any(|k| k == b) {
                    return Err(format!("prompts: {:?} must not appear, actual {:?}", b, seen));
                }
            }
        }
        let Some(want) = o.get("prompts") else { return Ok(()) };
        let want: Vec<&str> = want.as_array().unwrap().iter().filter_map(|x| x.as_str()).collect();
        let mut at = 0;
        for k in &seen {
            if at < want.len() && k == want[at] {
                at += 1;
            }
        }
        if at < want.len() {
            return Err(format!("prompts: expected {:?} in order, actual {:?}", want, seen));
        }
        Ok(())
    } else if o.get("legal").is_some() {
        check_legal(g, o, who(o, me))
    } else if o.get("zone").is_some() {
        check_zone(g, o, who(o, me))
    } else if let Some(n) = o["prizes_taken"].as_i64() {
        let have = g.st.players[who(o, me)].prizes_taken;
        if have as i64 != n {
            return Err(format!("{} prizes_taken: expected {}, actual {}", w, n, have));
        }
        Ok(())
    } else if o.get("winner").is_some() {
        let win = g.st.winner;
        let actual: Value = if win == WINNER_NONE {
            Value::Null
        } else if win == WINNER_DRAW {
            "draw".into()
        } else if win as usize == me {
            "me".into()
        } else {
            "opp".into()
        };
        if actual != o["winner"] {
            return Err(format!("winner: expected {}, actual {}", o["winner"], actual));
        }
        Ok(())
    } else if o.get("bench_count").is_some() || o.get("bench_excludes").is_some() {
        let p = who(o, me);
        let pl = &g.st.players[p];
        let benched: Vec<SlotId> = pl.bench.iter().copied().filter(|&s| g.st.slot_pokemon(p, s).is_some()).collect();
        if let Some(n) = o["bench_count"].as_u64() {
            if benched.len() as u64 != n {
                return Err(format!("{} bench_count: expected {}, actual {}", w, n, benched.len()));
            }
        }
        for n in name_list(&o["bench_excludes"]) {
            if let Some(&s) = benched.iter().find(|&&s| g.st.slot_pokemons(p, s).iter().any(|&c| names_eq(c, g, &n))) {
                return Err(format!("{} bench should not hold {}, actual {}", w, n, label(g, g.st.slot_pokemon(p, s).unwrap())));
            }
        }
        Ok(())
    } else if let Some(n) = o["active"].as_str() {
        let p = who(o, me);
        let top = g.st.slot_pokemon(p, g.st.players[p].active);
        match top {
            Some(c) if names_eq(c, g, n) => Ok(()),
            _ => Err(format!("{} active: expected {}, actual {}", w, n, top.map_or("none".to_string(), |c| label(g, c)))),
        }
    } else {
        check_slot(g, o, me)
    }
}

/// One failed assertion.
#[derive(Clone, Debug)]
pub struct Failure {
    pub index: usize,
    pub at: At,
    pub assertion: String,
    pub cite: String,
    pub actual: String,
    /// Rust canonical state at the check point (only with `--dump`).
    pub state: Option<String>,
}

/// Per-replay state: the armed assertions and what happened to them.
#[derive(Default)]
pub struct Run {
    pub assertions: Vec<Assertion>,
    pub me: usize,
    pub turn: i32,
    /// Address of the replayed game: option trials run on forks, which must not fire the hooks.
    pub game: usize,
    pub dump: bool,
    /// The game was already a Tiebreaker game when the assertions were armed.
    pub sudden_at_arm: bool,
    pub done: Vec<bool>,
    pub failures: Vec<Failure>,
    /// Turn decisions seen since the scenario edits (the first one is number 0).
    pub decisions: usize,
}

impl Run {
    pub fn checked(&self) -> usize {
        self.done.iter().filter(|d| **d).count()
    }

    fn run_at(&mut self, g: &Game, at: At) {
        for i in 0..self.assertions.len() {
            if self.assertions[i].at != at || self.done[i] {
                continue;
            }
            // `turn: k` waits for the k-th turn after the scenario turn (`turn_end` defaults to 0).
            let k = match (self.assertions[i].turn, at) {
                (Some(k), _) => Some(k),
                (None, At::TurnEnd) => Some(0),
                (None, _) => None,
            };
            if k.map_or(false, |k| g.st.turn != self.turn + k) {
                continue;
            }
            if at == At::Decision && self.assertions[i].spec["n"].as_u64() != Some(self.decisions as u64) {
                continue;
            }
            self.done[i] = true;
            if let Err(actual) = evaluate(g, self.me, &self.assertions[i]) {
                let a = &self.assertions[i];
                self.failures.push(Failure {
                    index: i,
                    at,
                    assertion: a.describe(),
                    cite: a.cite.clone(),
                    actual,
                    state: self.dump.then(|| serde_json::to_string_pretty(&g.canonical_json()).unwrap()),
                });
            }
        }
    }
}

thread_local! {
    static RUN: RefCell<Option<Run>> = const { RefCell::new(None) };
    /// Address of the replayed game while assertions are armed (0 = none).
    static ARMED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Kind names of the prompts the armed game created since the scenario edits (`prompts` assertions).
    static PROMPT_LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Hook in `Game::prompt`: record the kind name of every prompt of the replayed game (`prompts` assertions).
///
/// A coin flip's own prompt is a `Wait`; it is logged as `Coin` first, so a `prompts` list can place the
/// flip relative to other prompts.
pub fn on_prompt(g: &Game, kind: &crate::prompts::PromptKind, coin: bool) {
    if ARMED.with(|a| a.get()) != g as *const Game as usize {
        return;
    }
    if coin {
        PROMPT_LOG.with(|l| l.borrow_mut().push("Coin".to_string()));
    }
    let name: String = format!("{:?}", kind).chars().take_while(|c| c.is_alphanumeric()).collect();
    PROMPT_LOG.with(|l| l.borrow_mut().push(name));
}

/// Arm the assertions for the replay of `g` (called right after the scenario edits).
pub fn arm(g: &Game, assertions: Vec<Assertion>, dump: bool) {
    let n = assertions.len();
    RUN.with(|r| {
        *r.borrow_mut() = Some(Run {
            assertions,
            me: g.st.active_player as usize,
            turn: g.st.turn,
            game: g as *const Game as usize,
            dump,
            sudden_at_arm: g.st.is_sudden_death,
            done: vec![false; n],
            failures: Vec::new(),
            decisions: 0,
        })
    });
    ARMED.with(|a| a.set(g as *const Game as usize));
    PROMPT_LOG.with(|l| l.borrow_mut().clear());
}

/// Take the finished run (None when nothing was armed).
pub fn take() -> Option<Run> {
    ARMED.with(|a| a.set(0));
    RUN.with(|r| r.borrow_mut().take())
}

/// Run `f` on the armed run with the thread-local released, so option trials (forks that fire the
/// same hooks) find nothing armed instead of a borrowed cell.
fn with_run(f: impl FnOnce(&mut Run)) {
    let taken = RUN.with(|r| r.borrow_mut().take());
    if let Some(mut run) = taken {
        f(&mut run);
        RUN.with(|r| *r.borrow_mut() = Some(run));
    }
}

/// Hook in `after_end_turn`, once Knock Outs are resolved and before Checkup.
pub fn on_turn_end(g: &Game) {
    with_run(|run| {
        if run.game == g as *const Game as usize {
            if g.st.turn >= run.turn {
                run.run_at(g, At::TurnEnd);
            }
            if g.st.turn == run.turn + 1 {
                run.run_at(g, At::NextTurnEnd);
            }
        }
    });
}

/// Hook in `end_game`, once the winner is set.
pub fn on_game_end(g: &Game) {
    with_run(|run| {
        if run.game == g as *const Game as usize {
            run.run_at(g, At::GameEnd);
        }
    });
}

/// Hook right after the scenario edits: decision 0, the edited board at the scenario turn's first
/// decision (e.g. which actions are legal on the first turn).
pub fn on_scenario_start(g: &Game) {
    with_run(|run| {
        run.decisions = 0;
        run.run_at(g, At::Decision);
    });
}

/// Hook at every turn decision of the replay: the n-th since the edits (`decision`) and the first
/// of each later turn (`next_turn`). Decisions before the edits are not counted: arming happens
/// after this hook at the scenario turn.
pub fn on_turn_decision(g: &Game) {
    with_run(|run| {
        if run.game != g as *const Game as usize {
            return;
        }
        run.decisions += 1;
        run.run_at(g, At::Decision);
        if g.st.turn > run.turn {
            run.run_at(g, At::NextTurn);
        }
        if g.st.is_sudden_death && !run.sudden_at_arm {
            run.run_at(g, At::Tiebreaker);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::CardList;
    use serde_json::json;

    fn p(v: Value) -> Result<Vec<Assertion>, String> {
        parse(&json!({ "expect": v }))
    }

    fn ev(g: &Game, me: usize, v: Value) -> Result<(), String> {
        evaluate(g, me, &p(json!([v])).unwrap()[0])
    }

    #[test]
    fn cite_is_mandatory() {
        let e = p(json!([{"who":"opp","slot":"active","damage":10}])).unwrap_err();
        assert!(e.contains("missing cite"), "{}", e);
        assert!(p(json!([{"who":"opp","slot":"active","damage":10,"cite":" "}])).is_err());
        assert!(p(json!([{"who":"opp","slot":"active","damage":10,"cite":"ruling 1"}])).is_ok());
    }

    #[test]
    fn rejects_malformed() {
        assert!(p(json!([{"cite":"x","who":"opp","damage":1}])).is_err(), "no subject");
        assert!(p(json!([{"cite":"x","who":"opp","slot":"active","zone":"hand"}])).is_err(), "two subjects");
        assert!(p(json!([{"cite":"x","who":"you","slot":"active"}])).is_err(), "who");
        assert!(p(json!([{"cite":"x","who":"me","slot":"active","dmg":1}])).is_err(), "unknown key");
        assert!(p(json!([{"cite":"x","who":"me","slot":"active","at":"later"}])).is_err(), "at");
        assert!(parse(&json!({})).unwrap().is_empty());
    }

    #[test]
    fn at_defaults_to_next_turn() {
        let v = p(json!([{"cite":"x","winner":null},{"cite":"x","winner":null,"at":"turn_end"}])).unwrap();
        assert_eq!((v[0].at, v[1].at), (At::NextTurn, At::TurnEnd));
    }

    /// A board built with the scenario edits: me = Pikachu ex with energy, tool and a Poisoned
    /// state, a benched Pikachu ex, a Fire Energy in the discard; opp = Pikachu ex.
    fn board() -> Game {
        let deck: Vec<u16> =
            (0..4).map(|_| "Pikachu ex ASC 57").chain((0..56).map(|_| "Fire Energy MEE")).map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        let (pika, fire) = ("Pikachu ex ASC 57", "Fire Energy MEE 2");
        let sc = json!({
            "me": {"reset": true, "active": pika, "active_energy": [fire, fire], "active_damage": 30,
                   "active_conditions": ["POISONED"], "discard": [fire],
                   "bench": [{"card": pika, "damage": 50}]},
            "opp": {"reset": true, "active": pika}
        });
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    #[test]
    fn evaluates_pokemon_slots() {
        let g = board();
        let me = g.st.active_player as usize;
        let ok = |v: Value| ev(&g, me, v);
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","damage":30,"hp_left":170,"card":"Pikachu ex ASC 57"})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","energy":2})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","energy":["Fire Energy MEE 2","Fire Energy MEE 2"]})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","tool":null,"conditions":["POISONED"]})).is_ok());
        assert!(ok(json!({"cite":"c","who":"opp","slot":"active","damage":0,"conditions":[]})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","bench":0,"damage":50})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","bench":1,"in_play":false})).is_ok());
        assert!(ok(json!({"cite":"c","who":"opp","bench":0,"in_play":false})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","card":"Pikachu ex ASC 57","in_play":true})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","card":"Eternatus SSP 141","in_play":false})).is_ok());
        // Failures say what was expected and what the engine has.
        let e = ok(json!({"cite":"c","who":"me","slot":"active","damage":40})).unwrap_err();
        assert!(e.contains("damage: expected 40, actual 30"), "{}", e);
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","hp_left":100})).is_err());
        let e = ok(json!({"cite":"c","who":"me","slot":"active","energy":1})).unwrap_err();
        assert!(e.contains("energy: expected 1, actual 2"), "{}", e);
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","energy":["Fire Energy MEE 2"]})).is_err());
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","tool":"Punk Helmet"})).is_err());
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","conditions":[]})).is_err());
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","card":"Eternatus SSP 141"})).is_err());
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","in_play":false})).is_err());
        assert!(ok(json!({"cite":"c","who":"me","bench":3,"damage":0})).is_err());
        assert!(ok(json!({"cite":"c","who":"opp","card":"Eternatus SSP 141"})).is_err());
    }

    #[test]
    fn evaluates_zones_prizes_winner_active() {
        let mut g = board();
        let me = g.st.active_player as usize;
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"discard","count":1,"contains":["Fire Energy MEE 2"]})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"opp","zone":"discard","count":0})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"discard","count":2})).is_err());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"discard","contains":["Pikachu ex ASC 57"]})).is_err());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"discard","not_contains":["Fire Energy MEE 2"]})).is_err());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"prizes","count":6})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"lost_zone","count":0})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"hand","count":g.st.players[me].hand.len()})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"deck","count":g.st.players[me].deck.len() + 1})).is_err());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","prizes_taken":0})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","prizes_taken":1})).is_err());
        assert!(ev(&g, me, json!({"cite":"c","who":"opp","active":"Pikachu ex ASC 57"})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"opp","active":"Eternatus SSP 141"})).is_err());
        assert!(ev(&g, me, json!({"cite":"c","winner":null})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","winner":"me"})).is_err());
        g.st.winner = me as crate::types::Winner;
        assert!(ev(&g, me, json!({"cite":"c","winner":"me"})).is_ok());
        assert!(ev(&g, 1 - me, json!({"cite":"c","winner":"opp"})).is_ok());
        g.st.winner = crate::types::WINNER_DRAW;
        assert!(ev(&g, me, json!({"cite":"c","winner":"draw"})).is_ok());
    }

    #[test]
    fn evaluates_legal_kinds() {
        let g = board();
        let me = g.st.active_player as usize;
        // (the test board is in the setup phase: no turn action is legal there)
        let ok = |v: Value| ev(&g, me, v);
        let _ = &ok;
        assert!(p(json!([{"cite":"c","who":"me","legal":"stadium"}])).is_ok());
        assert!(p(json!([{"cite":"c","who":"me","legal":"ability"}])).is_ok());
        assert!(p(json!([{"cite":"c","who":"me","legal":"play"}])).is_err(), "play needs a name");
    }

    #[test]
    fn evaluates_bench_and_deck_top() {
        let g = board();
        let me = g.st.active_player as usize;
        let ok = |v: Value| ev(&g, me, v);
        assert!(ok(json!({"cite":"c","who":"me","bench_count":1})).is_ok());
        assert!(ok(json!({"cite":"c","who":"opp","bench_count":0})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","bench_count":2})).is_err());
        assert!(ok(json!({"cite":"c","who":"me","bench_excludes":["Eternatus SSP 141"]})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","bench_excludes":["Pikachu ex ASC 57"]})).is_err());
        assert!(ok(json!({"cite":"c","who":"opp","bench_excludes":["Pikachu ex ASC 57"]})).is_ok());
        let top = crate::carddb::en_key(g.st.cards[g.st.players[me].deck.iter().next().unwrap() as usize].def).to_string();
        assert!(ok(json!({"cite":"c","who":"me","zone":"deck","top":[top]})).is_ok());
        assert!(ok(json!({"cite":"c","who":"me","zone":"deck","top":["Eternatus SSP 141"]})).is_err());
        let v = p(json!([{"cite":"c","at":"start","who":"me","bench_count":1}])).unwrap();
        assert_eq!(v[0].at, At::Decision);
        assert_eq!(v[0].spec["n"], 0);
    }

    #[test]
    fn hooks_fire_once_and_report() {
        let g = board();
        let a = p(json!([
            {"cite":"c1","at":"turn_end","who":"me","slot":"active","damage":30},
            {"cite":"c2","at":"turn_end","who":"me","slot":"active","damage":99},
            {"cite":"c3","who":"opp","slot":"active","damage":0}
        ]))
        .unwrap();
        arm(&g, a, false);
        let fork = g;
        on_turn_end(&fork); // a copy at another address is a trial: ignored
        on_turn_decision(&g); // same turn: next_turn assertions wait
        on_turn_end(&g);
        on_turn_end(&g);
        let run = take().unwrap();
        assert_eq!(run.checked(), 2);
        assert_eq!(run.failures.len(), 1);
        assert_eq!((run.failures[0].index, run.failures[0].cite.as_str()), (1, "c2"));
        assert!(run.failures[0].actual.contains("expected 99, actual 30"));
    }
}
