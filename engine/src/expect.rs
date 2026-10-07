//! Scenario `expect` assertions: rules outcomes a scenario asserts about the
//! Rust engine's state (the oracle ignores the key). Evaluated by `diff` at
//! the end of the scenario turn (`turn_end`, after the attack and Knock Outs,
//! before Pokémon Checkup) or at the first turn decision of the next turn
//! (`next_turn`, after Checkup). Format: CARD_PORTING.md "Scenarios".

use crate::carddb::en_key;
use crate::engine::check::hp_of;
use crate::game::Game;
use crate::list::CardId;
use crate::state::SlotId;
use crate::types::{SpecialCondition, WINNER_DRAW, WINNER_NONE};
use serde_json::Value;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum At {
    TurnEnd,
    NextTurn,
    /// The end of the turn after the scenario turn (the opponent's turn), before its Pokémon Checkup.
    NextTurnEnd,
}

#[derive(Clone, Debug)]
pub struct Assertion {
    pub at: At,
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
    "not_contains", "prizes_taken", "winner", "active",
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
            Some(x) => return Err(format!("expect[{}]: bad at {} (turn_end, next_turn or next_turn_end)", i, x)),
        };
        let kinds = [
            o.contains_key("slot") || o.contains_key("bench") || (o.contains_key("card") && !o.contains_key("zone")),
            o.contains_key("zone"),
            o.contains_key("prizes_taken"),
            o.contains_key("winner"),
            o.contains_key("active"),
        ];
        if kinds.iter().filter(|k| **k).count() != 1 {
            return Err(format!("expect[{}]: needs exactly one subject (slot/bench/card, zone, prizes_taken, winner or active)", i));
        }
        if !o.contains_key("winner") && o.get("who").and_then(|w| w.as_str()).map_or(true, |w| w != "me" && w != "opp") {
            return Err(format!("expect[{}]: who must be me or opp", i));
        }
        out.push(Assertion { at, cite: cite.to_string(), spec: a.clone() });
    }
    Ok(out)
}

fn names_eq(c: CardId, g: &Game, name: &str) -> bool {
    let d = g.st.cards[c as usize].def;
    en_key(d) == name || crate::carddb::def(d).full_name == name || crate::carddb::en_name(d) == name
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
    for n in name_list(&a["not_contains"]) {
        if cards.iter().any(|&c| names_eq(c, g, &n)) {
            return Err(format!("{} {} should not contain {}, actual {}", w, zone, n, shown(g, &cards)));
        }
    }
    Ok(())
}

/// A `card` key selects the Pokémon when no `slot`/`bench` is given, else it checks the top Pokémon.
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
    if o.get("zone").is_some() {
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
    pub done: Vec<bool>,
    pub failures: Vec<Failure>,
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
            done: vec![false; n],
            failures: Vec::new(),
        })
    });
}

/// Take the finished run (None when nothing was armed).
pub fn take() -> Option<Run> {
    RUN.with(|r| r.borrow_mut().take())
}

/// Hook in `after_end_turn`, once Knock Outs are resolved and before Checkup.
pub fn on_turn_end(g: &Game) {
    RUN.with(|r| {
        if let Some(run) = r.borrow_mut().as_mut() {
            if run.game == g as *const Game as usize && g.st.turn == run.turn {
                run.run_at(g, At::TurnEnd);
            }
            if run.game == g as *const Game as usize && g.st.turn == run.turn + 1 {
                run.run_at(g, At::NextTurnEnd);
            }
        }
    });
}

/// Hook at every turn decision of the replay: the first one after the scenario turn.
pub fn on_turn_decision(g: &Game) {
    RUN.with(|r| {
        if let Some(run) = r.borrow_mut().as_mut() {
            if g.st.turn > run.turn {
                run.run_at(g, At::NextTurn);
            }
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
        // Scenario edits take Twinleaf full names (tools/check_cards.py maps English keys).
        let tw = |n: &str| crate::carddb::def(crate::carddb::def_by_full_name(n).unwrap()).full_name;
        let (pika, fire) = (tw("Pikachu ex ASC 57"), tw("Fire Energy MEE"));
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
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","energy":["Fire Energy MEE","Fire Energy MEE"]})).is_ok());
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
        assert!(ok(json!({"cite":"c","who":"me","slot":"active","energy":["Fire Energy MEE"]})).is_err());
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
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"discard","count":1,"contains":["Fire Energy MEE"]})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"opp","zone":"discard","count":0})).is_ok());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"discard","count":2})).is_err());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"discard","contains":["Pikachu ex ASC 57"]})).is_err());
        assert!(ev(&g, me, json!({"cite":"c","who":"me","zone":"discard","not_contains":["Fire Energy MEE"]})).is_err());
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
