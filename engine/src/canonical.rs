//! Canonical state serialization, identical to the oracle's
//! `oracle/canonical.ts`: sorted-key JSON, defaults omitted, cards as
//! `SET-NUMBER#id`, then 64-bit FNV-1a over the UTF-8 text.

use crate::carddb::def;
use crate::game::Game;
use crate::list::*;
use crate::markers::marker_name;
use crate::state::*;
use crate::types::*;
use serde_json::{json, Map, Value};

impl Game {
    fn refs_of(&self, cards: &[CardId]) -> Value {
        Value::Array(cards.iter().map(|c| Value::String(self.card_ref(*c))).collect())
    }

    fn sorted_refs(&self, cards: &[CardId]) -> Value {
        let mut v: Vec<String> = cards.iter().map(|c| self.card_ref(*c)).collect();
        v.sort();
        Value::Array(v.into_iter().map(Value::String).collect())
    }

    fn markers_json(&self, m: &Marker) -> Vec<Value> {
        let mut out: Vec<(String, Value)> = m
            .items
            .iter()
            .map(|it| {
                let mut o = Map::new();
                o.insert("name".into(), json!(marker_name(it.name)));
                if it.source != NO_CARD {
                    o.insert("source".into(), json!(self.card_ref(it.source)));
                }
                if let Some(s) = it.source_type.as_str() {
                    o.insert("sourceType".into(), json!(s));
                }
                if let Some(s) = it.target_scope.as_str() {
                    o.insert("targetScope".into(), json!(s));
                }
                let v = Value::Object(o);
                (serde_json::to_string(&v).unwrap(), v)
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out.into_iter().map(|x| x.1).collect()
    }

    fn slot_json(&self, s: &Slot) -> Value {
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
        nd!(hp, "hp");
        nd!(hp_bonus, "hpBonus");
        nd!(poison_damage, "poisonDamage");
        nd!(burn_damage, "burnDamage");
        nd!(confusion_damage, "confusionDamage");
        nd!(pokemon_played_turn, "pokemonPlayedTurn");
        nd!(ability_lock_activation_order, "abilityLockActivationOrder");
        nd!(sleep_flips, "sleepFlips");
        nd!(healed_this_turn, "healedThisTurn");
        nd!(cannot_be_healed_next_turn, "cannotBeHealedNextTurn");
        nd!(cannot_attack_next_turn, "cannotAttackNextTurn");
        nd!(cannot_attack_next_turn_pending, "cannotAttackNextTurnPending");
        nd!(cannot_retreat_next_turn, "cannotRetreatNextTurn");
        nd!(cannot_retreat_next_turn_pending, "cannotRetreatNextTurnPending");
        nd!(damage_reduction_next_turn, "damageReductionNextTurn");
        if !s.cannot_use_attacks_next_turn.is_empty() {
            o.insert("cannotUseAttacksNextTurn".into(), json!(s.cannot_use_attacks_next_turn.as_slice()));
        }
        if !s.cannot_use_attacks_next_turn_pending.is_empty() {
            o.insert("cannotUseAttacksNextTurnPending".into(), json!(s.cannot_use_attacks_next_turn_pending.as_slice()));
        }
        if !s.board_effect.is_empty() {
            o.insert("boardEffect".into(), json!(s.board_effect.as_slice()));
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
        let m = self.markers_json(&s.marker);
        if !m.is_empty() {
            o.insert("markers".into(), Value::Array(m));
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
        o.insert("active".into(), self.slot_json(&pl.slots[pl.active as usize]));
        o.insert("bench".into(), Value::Array(pl.bench.iter().map(|b| self.slot_json(&pl.slots[*b as usize])).collect()));
        let d = Player::new(pl.id);
        macro_rules! nd {
            ($field:ident, $key:expr) => {
                if pl.$field != d.$field {
                    o.insert($key.into(), json!(pl.$field));
                }
            };
        }
        nd!(supporter_turn, "supporterTurn");
        nd!(retreated_turn, "retreatedTurn");
        nd!(energy_played_turn, "energyPlayedTurn");
        nd!(stadium_played_turn, "stadiumPlayedTurn");
        nd!(stadium_used_turn, "stadiumUsedTurn");
        nd!(used_vstar, "usedVSTAR");
        nd!(used_gx, "usedGX");
        nd!(prizes_taken, "prizesTaken");
        nd!(prizes_taken_this_turn, "prizesTakenThisTurn");
        nd!(prizes_taken_last_turn, "prizesTakenLastTurn");
        nd!(pokemon_knocked_out_during_opponents_last_turn, "pokemonKnockedOutDuringOpponentsLastTurn");
        nd!(pokemon_knocked_out_by_attack_during_opponents_last_turn, "pokemonKnockedOutByAttackDuringOpponentsLastTurn");
        nd!(can_evolve, "canEvolve");
        nd!(ancient_pokemon_attacked_last_turn, "ancientPokemonAttackedLastTurn");
        nd!(cannot_play_item_cards, "cannotPlayItemCards");
        nd!(cannot_play_supporter_cards, "cannotPlaySupporterCards");
        nd!(cannot_play_stadium_cards, "cannotPlayStadiumCards");
        nd!(cannot_play_tool_cards, "cannotPlayToolCards");
        nd!(cannot_play_special_energy_cards, "cannotPlaySpecialEnergyCards");
        nd!(cannot_play_energy_cards, "cannotPlayEnergyCards");
        nd!(cannot_play_pokemon_cards, "cannotPlayPokemonCards");
        nd!(cannot_play_pokemon_with_abilities, "cannotPlayPokemonWithAbilities");
        nd!(cannot_evolve_pokemon_cards, "cannotEvolvePokemonCards");
        nd!(play_locks_turns_remaining, "playLocksTurnsRemaining");
        nd!(used_dragons_wish, "usedDragonsWish");
        nd!(unlimited_energy_attach_turns_remaining, "unlimitedEnergyAttachTurnsRemaining");
        nd!(cannot_draw_at_start_of_turn, "cannotDrawAtStartOfTurn");
        nd!(cannot_attack_turns_remaining, "cannotAttackTurnsRemaining");
        nd!(stadium_and_tool_have_no_effect_turns_remaining, "stadiumAndToolHaveNoEffectTurnsRemaining");
        nd!(coin_flip_cancel_trainer_play_turns_remaining, "coinFlipCancelTrainerPlayTurnsRemaining");
        nd!(used_table_turner, "usedTableTurner");
        nd!(chains_of_control_used, "chainsOfControlUsed");
        nd!(pecharuntex_is_in_play, "pecharuntexIsInPlay");
        nd!(used_run_errand, "usedRunErrand");
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
        let m = self.markers_json(&pl.marker);
        if !m.is_empty() {
            o.insert("markers".into(), Value::Array(m));
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
            if d.is_pokemon() {
                if inst.moved_to_active_this_turn {
                    diff.insert("movedToActiveThisTurn".into(), json!(true));
                }
                if inst.damage_taken_last_turn != 0 {
                    diff.insert("damageTakenLastTurn".into(), json!(inst.damage_taken_last_turn));
                }
            }
            if inst.extra_prizes {
                diff.insert("extraPrizes".into(), json!(true));
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
                    st.players[p].id.to_string(),
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
