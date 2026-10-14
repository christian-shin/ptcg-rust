//! Canonical state serialization: sorted-key JSON, defaults omitted, cards as
//! `SET-NUMBER#id`, then 64-bit FNV-1a over the UTF-8 text. The recorded traces hold this form.

use crate::carddb::def;
use crate::game::Game;
use crate::list::*;
use crate::state::*;
use crate::types::*;
use serde_json::{json, Map, Value};

/// A player's number in the recorded traces and the canonical JSON: the player index plus one (1 or 2). The engine
/// itself names a player by index everywhere; only the trace boundary writes this number.
fn wire_player(p: usize) -> u8 {
    p as u8 + 1
}

impl Game {
    fn refs_of(&self, cards: &[CardId]) -> Value {
        Value::Array(cards.iter().map(|c| Value::String(self.card_ref(*c))).collect())
    }

    fn sorted_refs(&self, cards: &[CardId]) -> Value {
        let mut v: Vec<String> = cards.iter().map(|c| self.card_ref(*c)).collect();
        v.sort();
        Value::Array(v.into_iter().map(Value::String).collect())
    }

    /// The lasting effects on `on`, in the order they were put: what each is, its timing and the card that left it.
    fn lasting_json(&self, on: LastingOn) -> Vec<Value> {
        let attack = |a: AttackRef| json!(self.st.cdef(a.card).attacks[a.idx()].name);
        let player = |p: u8| json!(wire_player(p as usize));
        self.st
            .lasting
            .iter()
            .filter(|l| l.on == on)
            .map(|l| {
                let mut o = Map::new();
                o.insert("source".into(), json!(self.card_ref(l.source)));
                match l.what {
                    LastingWhat::Lock { decl, turns, attack: a } => {
                        o.insert("lock".into(), json!(decl.error));
                        o.insert("turns".into(), json!(turns));
                        if let Some(a) = a {
                            o.insert("attack".into(), attack(a));
                        }
                    }
                    LastingWhat::PlayerLock { decl, turns, .. } => {
                        o.insert("lock".into(), json!(decl.error));
                        o.insert("turns".into(), json!(turns));
                    }
                    LastingWhat::Recorded => {
                        o.insert("recorded".into(), json!(true));
                    }
                    LastingWhat::Declared { index, until } => {
                        o.insert("declared".into(), json!(index));
                        match until {
                            crate::state::Until::ThisTurn(p) => o.insert("thisTurnOf".into(), player(p)),
                            crate::state::Until::Removed => o.insert("untilRemoved".into(), json!(true)),
                        };
                    }
                    LastingWhat::Spawn { ends_with, .. } => {
                        o.insert("newPokemon".into(), json!(true));
                        o.insert("endsWith".into(), player(ends_with));
                    }
                    LastingWhat::TakesLessDamage(n) => {
                        o.insert("takesLessDamage".into(), json!(n));
                    }
                    LastingWhat::DealsLessDamage(n) => {
                        o.insert("dealsLessDamage".into(), json!(n));
                    }
                    LastingWhat::TakesMoreDamage { amount, attacker, pending, rearm } => {
                        o.insert("takesMoreDamage".into(), json!(amount));
                        if let Some(p) = attacker {
                            o.insert("attacker".into(), player(p));
                        }
                        o.insert("pending".into(), json!(pending));
                        o.insert("rearm".into(), json!(rearm));
                    }
                    LastingWhat::Prevent { spec, pending } => {
                        o.insert("prevent".into(), json!(spec));
                        o.insert("pending".into(), json!(pending));
                    }
                    LastingWhat::Retaliate { amount, attack: a, attacker, pending } => {
                        o.insert("retaliate".into(), json!(amount));
                        o.insert("attack".into(), attack(a));
                        o.insert("attacker".into(), player(attacker));
                        o.insert("pending".into(), json!(pending));
                    }
                    LastingWhat::DiscardAttackerEnergy { attack: a, attacker, armed, pending } => {
                        o.insert("discardAttackerEnergy".into(), json!(armed));
                        o.insert("attack".into(), attack(a));
                        o.insert("attacker".into(), player(attacker));
                        o.insert("pending".into(), json!(pending));
                    }
                    LastingWhat::AttackCostIncrease { now, pending, attacker } | LastingWhat::RetreatCostIncrease { now, pending, attacker } => {
                        let key = if matches!(l.what, LastingWhat::AttackCostIncrease { .. }) { "attackCostIncrease" } else { "retreatCostIncrease" };
                        o.insert(key.into(), json!(now));
                        o.insert("pendingIncrease".into(), json!(pending));
                        if let Some(p) = attacker {
                            o.insert("attacker".into(), player(p));
                        }
                    }
                    LastingWhat::NoWeakness { armed, pending } => {
                        o.insert("noWeakness".into(), json!(armed));
                        o.insert("pending".into(), json!(pending));
                    }
                    LastingWhat::NextTurnBonus { spec, attack: a, pending } => {
                        o.insert("nextTurnBonus".into(), json!(spec.bonus));
                        o.insert("attack".into(), attack(a));
                        o.insert("pending".into(), json!(pending));
                    }
                }
                Value::Object(o)
            })
            .collect()
    }

    /// The usage records (`State::usage`) as text, by the Ability's printed name or the declaring card's key and
    /// declaration index (no process-local numbers), sorted so the text is deterministic.
    fn usage_json(&self) -> Vec<Value> {
        use crate::state::{Until, UsageBy, UsageKey};
        let mut out: Vec<(String, Value)> = self
            .st
            .usage
            .iter()
            .map(|u| {
                let mut o = Map::new();
                match u.key {
                    UsageKey::Ability(n) => o.insert("ability".into(), json!(crate::gen::cards::ABILITY_NAMES[n as usize])),
                    UsageKey::Declared { def: d, index } => o.insert("declared".into(), json!(format!("{}#{}", def(d).full_name, index))),
                };
                match u.by {
                    UsageBy::Player(p) => o.insert("player".into(), json!(wire_player(p as usize))),
                    UsageBy::Card(c) => o.insert("card".into(), json!(self.card_ref(c))),
                };
                match u.until {
                    Until::ThisTurn(p) => o.insert("thisTurnOf".into(), json!(wire_player(p as usize))),
                    Until::Removed => o.insert("untilRemoved".into(), json!(true)),
                };
                let v = Value::Object(o);
                (serde_json::to_string(&v).unwrap(), v)
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out.into_iter().map(|(_, v)| v).collect()
    }

    fn slot_json(&self, p: usize, si: SlotId) -> Value {
        let s = &self.st.players[p].slots[si as usize];
        let mut o = Map::new();
        o.insert("cards".into(), self.refs_of(s.cards.as_slice()));
        let d = Slot::default();
        macro_rules! nd {
            ($field:ident, $key:expr) => {
                if s.$field != d.$field {
                    o.insert($key.into(), json!(s.$field));
                }
            };
        }
        nd!(damage, "damage");
        nd!(poison_damage, "poisonDamage");
        nd!(burn_damage, "burnDamage");
        nd!(confusion_damage, "confusionDamage");
        nd!(entered_turn, "pokemonPlayedTurn");
        nd!(sleep_flips, "sleepFlips");
        nd!(healed_this_turn, "healedThisTurn");
        // The lasting effects attacks left on the Pokémon (the lasting table, outside the observable projection).
        let lasting = self.lasting_json(LastingOn::pokemon(p, si));
        if !lasting.is_empty() {
            o.insert("lasting".into(), Value::Array(lasting));
        }
        if let Some(a) = s.attacks_this_turn {
            if a != 0 {
                o.insert("attacksThisTurn".into(), json!(a));
            }
        }
        if !s.energies.is_empty() {
            o.insert("energies".into(), self.refs_of(s.energies.as_slice()));
        }
        if !s.tools.is_empty() {
            o.insert("tools".into(), self.refs_of(s.tools.as_slice()));
        }
        if !s.special_conditions.is_empty() {
            let mut v: Vec<u8> = s.special_conditions.as_slice().to_vec();
            v.sort();
            o.insert("specialConditions".into(), json!(v));
        }
        Value::Object(o)
    }

    fn player_json(&self, p: usize) -> Value {
        let pl = &self.st.players[p];
        let mut o = Map::new();
        o.insert("deck".into(), self.refs_of(pl.deck.as_slice()));
        o.insert("hand".into(), self.sorted_refs(pl.hand.as_slice()));
        o.insert("discard".into(), self.sorted_refs(pl.discard.as_slice()));
        o.insert("lostzone".into(), self.sorted_refs(pl.lostzone.as_slice()));
        o.insert("stadium".into(), self.refs_of(pl.stadium.as_slice()));
        o.insert("supporter".into(), self.refs_of(pl.supporter.as_slice()));
        o.insert(
            "prizes".into(),
            Value::Array((0..pl.prize_count as usize).map(|i| self.refs_of(pl.prizes[i].as_slice())).collect()),
        );
        if pl.prize_face_up[..pl.prize_count as usize].iter().any(|v| *v) {
            o.insert("faceUpPrizes".into(), json!(pl.prize_face_up[..pl.prize_count as usize].to_vec()));
        }
        o.insert("active".into(), self.slot_json(p, pl.active));
        o.insert("bench".into(), Value::Array(pl.bench.iter().map(|b| self.slot_json(p, *b)).collect()));
        let d = Player::new();
        macro_rules! nd {
            ($field:ident, $key:expr) => {
                if pl.$field != d.$field {
                    o.insert($key.into(), json!(pl.$field));
                }
            };
        }
        // The played-this-turn record (events batch 7), as the keys the state always had (none is observable).
        let played = |f: &dyn Fn(&crate::carddb::CardDef) -> bool| pl.played_this_turn.iter().filter(|c| f(self.st.cdef(**c))).count();
        let supporters = played(&|d| d.trainer_type == crate::types::TrainerType::Supporter as u8);
        if supporters > 0 {
            o.insert("supporterTurn".into(), json!(supporters));
        }
        nd!(retreated_turn, "retreatedTurn");
        nd!(energy_played_turn, "energyPlayedTurn");
        if played(&|d| d.trainer_type == crate::types::TrainerType::Stadium as u8) > 0 {
            o.insert("stadiumPlayedTurn".into(), json!(self.st.turn));
        }
        nd!(stadium_used_turn, "stadiumUsedTurn");
        nd!(used_vstar, "usedVSTAR");
        nd!(used_gx, "usedGX");
        nd!(prizes_taken, "prizesTaken");
        nd!(prizes_taken_this_turn, "prizesTakenThisTurn");
        nd!(prizes_taken_last_turn, "prizesTakenLastTurn");
        nd!(pokemon_knocked_out_during_opponents_last_turn, "pokemonKnockedOutDuringOpponentsLastTurn");
        nd!(pokemon_knocked_out_by_attack_during_opponents_last_turn, "pokemonKnockedOutByAttackDuringOpponentsLastTurn");
        if played(&|d| d.trainer_type == crate::types::TrainerType::Supporter as u8 && d.has_tag(crate::types::tag::ANCIENT)) > 0 {
            o.insert("ancientSupporter".into(), json!(true));
        }
        // The locks an attack left on the player (the lasting table, outside the observable projection).
        let lasting = self.lasting_json(LastingOn::Player(p as u8));
        if !lasting.is_empty() {
            o.insert("lasting".into(), Value::Array(lasting));
        }
        if played(&|d| d.trainer_type == crate::types::TrainerType::Supporter as u8 && d.has_tag(crate::types::tag::TEAM_ROCKET)) > 0 {
            o.insert("rocketSupporter".into(), json!(true));
        }
        if !pl.moved_to_active_this_turn.is_empty() {
            o.insert("movedToActiveThisTurn".into(), json!(pl.moved_to_active_this_turn.as_slice()));
        }
        if !pl.moved_from_active_to_bench_this_turn.is_empty() {
            o.insert("movedFromActiveToBenchThisTurn".into(), json!(pl.moved_from_active_to_bench_this_turn.as_slice()));
        }
        if !pl.pokemon_knocked_out_last_turn_entries.is_empty() {
            let v: Vec<Value> = pl.pokemon_knocked_out_last_turn_entries.iter().map(|d| json!(def(*d).tag_names)).collect();
            o.insert("pokemonKnockedOutLastTurnEntries".into(), Value::Array(v));
        }
        Value::Object(o)
    }

    fn card_mutations(&self) -> Map<String, Value> {
        let mut out = Map::new();
        let mut seen = [false; MAX_CARDS];
        let mut visit = |c: CardId, out: &mut Map<String, Value>| {
            if seen[c as usize] {
                return;
            }
            seen[c as usize] = true;
            let inst = &self.st.cards[c as usize];
            let d = self.st.cdef(c);
            let mut diff = Map::new();
            // Fossils (Antique Root Fossil) record that they moved to the Active Spot this turn but not the damage
            // they took last turn.
            if crate::carddb::pokemon_flags(inst.def) & crate::carddb::PF_FOSSIL != 0 && !d.is_pokemon() && inst.moved_to_active_this_turn {
                diff.insert("movedToActiveThisTurn".into(), json!(true));
            }
            if d.is_pokemon() {
                if inst.moved_to_active_this_turn {
                    diff.insert("movedToActiveThisTurn".into(), json!(true));
                }
                if inst.damage_taken_last_turn != 0 {
                    diff.insert("damageTakenLastTurn".into(), json!(inst.damage_taken_last_turn));
                }
            }
            if !diff.is_empty() {
                out.insert(self.card_ref(c), Value::Object(diff));
            }
        };
        for p in 0..2 {
            let pl = &self.st.players[p];
            let lists: Vec<&[CardId]> = {
                let mut v: Vec<&[CardId]> = vec![
                    pl.deck.as_slice(),
                    pl.hand.as_slice(),
                    pl.discard.as_slice(),
                    pl.lostzone.as_slice(),
                    pl.stadium.as_slice(),
                    pl.supporter.as_slice(),
                    pl.slots[pl.active as usize].cards.as_slice(),
                ];
                for b in pl.bench.iter() {
                    v.push(pl.slots[*b as usize].cards.as_slice());
                }
                for i in 0..pl.prize_count as usize {
                    v.push(pl.prizes[i].as_slice());
                }
                v.push(pl.slots[pl.active as usize].tools.as_slice());
                for b in pl.bench.iter() {
                    v.push(pl.slots[*b as usize].tools.as_slice());
                }
                v
            };
            for l in lists {
                for &c in l {
                    visit(c, &mut out);
                }
            }
        }
        out
    }

    pub fn canonical_json(&self) -> Value {
        let st = &self.st;
        let mut o = Map::new();
        o.insert("phase".into(), json!(st.phase as u8));
        o.insert("turn".into(), json!(st.turn));
        o.insert("activePlayer".into(), json!(st.active_player));
        o.insert("winner".into(), json!(st.winner));
        o.insert("abilityLockOrderCounter".into(), json!(st.ability_lock_order_counter));
        if st.skip_opponent_turn {
            o.insert("skipOpponentTurn".into(), json!(true));
        }
        if let Some(a) = st.last_attack {
            o.insert("lastAttack".into(), json!(st.cdef(a.card).attacks[a.idx()].name));
        }
        let mut pla = Map::new();
        for p in 0..2 {
            if let Some((a, src)) = st.player_last_attack[p] {
                pla.insert(
                    wire_player(p).to_string(),
                    json!({ "attack": st.cdef(a.card).attacks[a.idx()].name, "sourceCard": self.card_ref(src) }),
                );
            }
        }
        if !pla.is_empty() {
            o.insert("playerLastAttack".into(), Value::Object(pla));
        }
        if st.is_sudden_death {
            o.insert("isSuddenDeath".into(), json!(true));
        }
        if st.bench_size_change_handled {
            o.insert("benchSizeChangeHandled".into(), json!(true));
        }
        o.insert("players".into(), Value::Array((0..st.players_added as usize).map(|p| self.player_json(p)).collect()));
        let usage = self.usage_json();
        if !usage.is_empty() {
            o.insert("usage".into(), Value::Array(usage));
        }
        let cards = self.card_mutations();
        if !cards.is_empty() {
            o.insert("cards".into(), Value::Object(cards));
        }
        Value::Object(o)
    }

    pub fn canonical_text(&self) -> String {
        serde_json::to_string(&self.canonical_json()).unwrap()
    }

    pub fn state_hash(&self) -> String {
        fnv1a64(self.canonical_text().as_bytes())
    }
}

/// Slot fields a player can observe: the Pokémon, its attachments, damage and conditions.
const OBSERVABLE_SLOT_KEYS: [&str; 7] = ["cards", "energies", "tools", "damage", "specialConditions", "poisonDamage", "burnDamage"];
/// Player zones a player can observe (the deck as a set: its order is hidden).
const OBSERVABLE_PLAYER_KEYS: [&str; 7] = ["hand", "discard", "lostzone", "stadium", "supporter", "prizes", "faceUpPrizes"];
const OBSERVABLE_STATE_KEYS: [&str; 4] = ["phase", "turn", "activePlayer", "winner"];

/// The player-observable projection of a canonical state: what
/// the observable comparator compares.
pub fn observable_json(can: &Value) -> Value {
    fn pick(src: &Value, keys: &[&str], out: &mut Map<String, Value>) {
        for k in keys {
            if let Some(v) = src.get(*k) {
                out.insert((*k).into(), v.clone());
            }
        }
    }
    let slot = |s: &Value| {
        let mut o = Map::new();
        pick(s, &OBSERVABLE_SLOT_KEYS, &mut o);
        Value::Object(o)
    };
    let mut out = Map::new();
    pick(can, &OBSERVABLE_STATE_KEYS, &mut out);
    let players: Vec<Value> = can["players"]
        .as_array()
        .map(|ps| {
            ps.iter()
                .map(|p| {
                    let mut o = Map::new();
                    let mut deck: Vec<String> = p["deck"].as_array().map(|d| d.iter().filter_map(|c| c.as_str().map(String::from)).collect()).unwrap_or_default();
                    deck.sort();
                    o.insert("deck".into(), json!(deck));
                    o.insert("active".into(), slot(&p["active"]));
                    o.insert("bench".into(), Value::Array(p["bench"].as_array().map(|b| b.iter().map(slot).collect()).unwrap_or_default()));
                    pick(p, &OBSERVABLE_PLAYER_KEYS, &mut o);
                    Value::Object(o)
                })
                .collect()
        })
        .unwrap_or_default();
    out.insert("players".into(), Value::Array(players));
    Value::Object(out)
}

impl Game {
    pub fn observable_hash(&self) -> String {
        fnv1a64(serde_json::to_string(&observable_json(&self.canonical_json())).unwrap().as_bytes())
    }
}

pub fn fnv1a64(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:016x}", h)
}

pub fn _phase_name(p: GamePhase) -> u8 {
    p as u8
}

