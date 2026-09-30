//! Prompts: the decision taxonomy (Twinleaf `game/store/prompts`).
//!
//! `describe` emits the same canonical descriptor as the oracle's
//! `describePrompt`; `decode` turns a raw wire answer into a resolved value,
//! applying the same validation as `prompt.decode` + `prompt.validate`.

use crate::carddb::CardDef;
use crate::effects::{Cost, EnergyEntry, EnergyMap, SlotRef};
use crate::energy;
use crate::game::{Game, GameError};
use crate::list::*;
use crate::state::*;
use crate::types::*;
use serde_json::{json, Value};

/// Partial-card filter (`FilterType`), keys in insertion order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Filter {
    pub super_type: Option<u8>,
    pub stage: Option<u8>,
    pub trainer_type: Option<u8>,
    pub energy_type: Option<u8>,
    pub card_type: Option<CardType>,
    pub tags: Option<u32>,
    pub name: Option<&'static str>,
}

impl Filter {
    pub fn none() -> Filter {
        Filter::default()
    }
    pub fn super_type(s: SuperType) -> Filter {
        Filter { super_type: Some(s as u8), ..Default::default() }
    }
    /// `matchesPromptFilter`.
    pub fn matches(&self, d: &CardDef) -> bool {
        if let Some(v) = self.super_type {
            if d.super_type != v {
                return false;
            }
        }
        if let Some(v) = self.stage {
            if !d.is_pokemon() || d.stage != v {
                return false;
            }
        }
        if let Some(v) = self.trainer_type {
            if !d.is_trainer() || d.trainer_type != v {
                return false;
            }
        }
        if let Some(v) = self.energy_type {
            // Pokémon and Trainers have no energyType (undefined !== v).
            if !d.is_energy() || d.energy_type != v {
                return false;
            }
        }
        if let Some(t) = self.card_type {
            if d.is_energy() {
                if !d.provides.contains(&t) {
                    return false;
                }
            } else if d.is_pokemon() {
                if !d.card_type.contains(&t) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(t) = self.tags {
            if !d.has_tag(t) {
                return false;
            }
        }
        if let Some(n) = self.name {
            if d.name != n {
                return false;
            }
        }
        true
    }
    pub fn to_json(&self) -> Value {
        let mut m = serde_json::Map::new();
        if let Some(v) = self.super_type {
            m.insert("superType".into(), json!(v));
        }
        if let Some(v) = self.stage {
            m.insert("stage".into(), json!(v));
        }
        if let Some(v) = self.trainer_type {
            m.insert("trainerType".into(), json!(v));
        }
        if let Some(v) = self.energy_type {
            m.insert("energyType".into(), json!(v));
        }
        if let Some(v) = self.card_type {
            m.insert("cardType".into(), json!(v));
        }
        if let Some(v) = self.tags {
            m.insert("tags".into(), json!([TAG_NAMES[v as usize]]));
        }
        if let Some(v) = self.name {
            m.insert("name".into(), json!(v));
        }
        Value::Object(m)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChooseCardsOpts {
    pub min: u8,
    pub max: u8,
    pub allow_cancel: bool,
    pub blocked: SVec<u8, 60>,
    pub is_secret: bool,
    pub different_types: bool,
    pub allow_different_super_types: bool,
    pub max_pokemons: Option<u8>,
    pub max_basic_energies: Option<u8>,
    pub max_energies: Option<u8>,
    pub max_trainers: Option<u8>,
    pub max_tools: Option<u8>,
    pub max_stadiums: Option<u8>,
    pub max_supporters: Option<u8>,
    pub max_special_energies: Option<u8>,
    pub max_items: Option<u8>,
    pub max_basics: Option<u8>,
    pub max_evolutions: Option<u8>,
}

impl ChooseCardsOpts {
    pub fn new(min: u8, max: u8, allow_cancel: bool) -> Self {
        ChooseCardsOpts {
            min,
            max,
            allow_cancel,
            blocked: SVec::new(),
            is_secret: false,
            different_types: false,
            allow_different_super_types: true,
            max_pokemons: None,
            max_basic_energies: None,
            max_energies: None,
            max_trainers: None,
            max_tools: None,
            max_stadiums: None,
            max_supporters: None,
            max_special_energies: None,
            max_items: None,
            max_basics: None,
            max_evolutions: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SelectValues {
    /// "Draw N card(s)" for N from `max` down to 0 (mulligan / DRAW_UP_TO_X_CARDS).
    DrawCards(u8),
    Static(&'static [&'static str]),
}

impl SelectValues {
    pub fn len(&self) -> usize {
        match self {
            SelectValues::DrawCards(m) => *m as usize + 1,
            SelectValues::Static(v) => v.len(),
        }
    }
    pub fn values(&self) -> Vec<String> {
        match self {
            SelectValues::DrawCards(m) => (0..=*m).rev().map(|i| format!("Draw {} card(s)", i)).collect(),
            SelectValues::Static(v) => v.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PromptKind {
    // chance
    ShuffleDeck,
    CoinFlip,
    // info
    Alert,
    ShowCards,
    ShowMulligan,
    ConfirmCards,
    Wait,
    // decisions
    Confirm,
    ChooseCards { cards: ListRef, filter: Filter, opts: ChooseCardsOpts },
    ChoosePokemon { player_type: PlayerType, slots: SVec<u8, 3>, min: u8, max: u8, allow_cancel: bool, blocked: SVec<CardTarget, 8> },
    ChoosePrize { count: u8, blocked: SVec<u8, 6>, use_opponent_prizes: bool, allow_cancel: bool, is_secret: bool, destination: Option<ListRef> },
    Select { values: SelectValues, allow_cancel: bool, default_value: i32 },
    ChooseEnergy { energy: EnergyMap, cost: Cost, allow_cancel: bool },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PromptClass {
    Chance,
    Info,
    Decision,
}

#[derive(Clone, Copy, Debug)]
pub struct PromptRec {
    pub id: u32,
    /// Answering player's id.
    pub player_id: u8,
    pub perspective: Option<u8>,
    pub message: &'static str,
    pub kind: PromptKind,
    pub result: Option<Res>,
}

/// Decoded prompt result (what Twinleaf stores in `prompt.result`).
#[derive(Clone, Copy, Debug)]
pub enum Res {
    Null,
    True,
    Bool(bool),
    Int(i32),
    Cards(SVec<CardId, 16>),
    Slots(SVec<SlotRef, 8>),
    Energy(SVec<EnergyEntry, 40>),
    /// Indices into the owning player's `prizes` array.
    Prizes(SVec<u8, 6>),
    Order(List<60>),
}

impl Res {
    pub fn is_null(&self) -> bool {
        matches!(self, Res::Null)
    }
    pub fn as_bool(&self) -> bool {
        matches!(self, Res::Bool(true) | Res::True)
    }
    pub fn as_int(&self) -> i32 {
        match self {
            Res::Int(v) => *v,
            _ => 0,
        }
    }
    pub fn cards(&self) -> &[CardId] {
        match self {
            Res::Cards(c) => c.as_slice(),
            _ => &[],
        }
    }
    pub fn slots(&self) -> &[SlotRef] {
        match self {
            Res::Slots(s) => s.as_slice(),
            _ => &[],
        }
    }
}

impl PromptRec {
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            PromptKind::ShuffleDeck => "Shuffle deck",
            PromptKind::CoinFlip => "Coin flip",
            PromptKind::Alert => "Alert",
            PromptKind::ShowCards => "Show cards",
            PromptKind::ShowMulligan => "Show mulligan",
            PromptKind::ConfirmCards => "Confirm cards",
            PromptKind::Wait => "WaitPrompt",
            PromptKind::Confirm => "Confirm",
            PromptKind::ChooseCards { .. } => "Choose cards",
            PromptKind::ChoosePokemon { .. } => "Choose pokemon",
            PromptKind::ChoosePrize { .. } => "Choose prize",
            PromptKind::Select { .. } => "Select",
            PromptKind::ChooseEnergy { .. } => "Choose energy",
        }
    }
    pub fn class_name(&self) -> &'static str {
        match self.kind {
            PromptKind::ShuffleDeck => "ShuffleDeckPrompt",
            PromptKind::CoinFlip => "CoinFlipPrompt",
            PromptKind::Alert => "AlertPrompt",
            PromptKind::ShowCards => "ShowCardsPrompt",
            PromptKind::ShowMulligan => "ShowMulliganPrompt",
            PromptKind::ConfirmCards => "ConfirmCardsPrompt",
            PromptKind::Wait => "WaitPrompt",
            PromptKind::Confirm => "ConfirmPrompt",
            PromptKind::ChooseCards { .. } => "ChooseCardsPrompt",
            PromptKind::ChoosePokemon { .. } => "ChoosePokemonPrompt",
            PromptKind::ChoosePrize { .. } => "ChoosePrizePrompt",
            PromptKind::Select { .. } => "SelectPrompt",
            PromptKind::ChooseEnergy { .. } => "ChooseEnergyPrompt",
        }
    }
    pub fn class(&self) -> PromptClass {
        match self.kind {
            PromptKind::ShuffleDeck | PromptKind::CoinFlip => PromptClass::Chance,
            PromptKind::Alert | PromptKind::ShowCards | PromptKind::ShowMulligan | PromptKind::ConfirmCards | PromptKind::Wait => PromptClass::Info,
            _ => PromptClass::Decision,
        }
    }
    pub fn perspective_id(&self) -> u8 {
        self.perspective.unwrap_or(self.player_id)
    }
}

// ---------------------------------------------------------------------------
// Target helpers

/// Resolve a `CardTarget` relative to player `p` (`StateUtils.getTarget`).
pub fn get_target(st: &State, p: usize, t: CardTarget) -> Result<SlotRef, GameError> {
    let q = if t.player == PlayerType::TopPlayer { 1 - p } else { p };
    match t.slot {
        SlotType::Active => Ok(SlotRef::new(q, st.players[q].active)),
        SlotType::Bench => match st.players[q].bench.get(t.index as usize) {
            Some(s) => Ok(SlotRef::new(q, *s)),
            None => Err(GameError("INVALID_TARGET")),
        },
        _ => Err(GameError("INVALID_TARGET")),
    }
}

/// In-play targets for (playerType, slots) relative to `p`, in oracle order.
pub fn slot_targets(st: &State, p: usize, pt: PlayerType, slots: &[u8]) -> Vec<CardTarget> {
    let mut out = Vec::new();
    let sides: &[PlayerType] = match pt {
        PlayerType::Any => &[PlayerType::BottomPlayer, PlayerType::TopPlayer],
        PlayerType::BottomPlayer => &[PlayerType::BottomPlayer],
        PlayerType::TopPlayer => &[PlayerType::TopPlayer],
    };
    for &side in sides {
        let q = if side == PlayerType::BottomPlayer { p } else { 1 - p };
        let pl = &st.players[q];
        if slots.contains(&(SlotType::Active as u8)) && !pl.slots[pl.active as usize].cards.is_empty() {
            out.push(CardTarget::new(side, SlotType::Active, 0));
        }
        if slots.contains(&(SlotType::Bench as u8)) {
            for (i, &b) in pl.bench.iter().enumerate() {
                if !pl.slots[b as usize].cards.is_empty() {
                    out.push(CardTarget::new(side, SlotType::Bench, i as u8));
                }
            }
        }
    }
    out
}

pub fn target_json(t: CardTarget) -> Value {
    json!({ "player": t.player as u8, "slot": t.slot as u8, "index": t.index })
}

pub fn target_from_json(v: &Value) -> Option<CardTarget> {
    Some(CardTarget::new(
        PlayerType::from_u8(v.get("player")?.as_u64()? as u8),
        SlotType::from_u8(v.get("slot")?.as_u64()? as u8),
        v.get("index")?.as_u64()? as u8,
    ))
}

fn blocked_slots(st: &State, p: usize, blocked: &[CardTarget]) -> Vec<SlotRef> {
    blocked.iter().filter_map(|b| get_target(st, p, *b).ok()).collect()
}

// ---------------------------------------------------------------------------
// Descriptors

impl Game {
    pub fn card_ref(&self, c: CardId) -> String {
        let d = self.st.cdef(c);
        format!("{}-{}#{}", d.set, d.set_number, c)
    }

    fn refs(&self, cards: &[CardId]) -> Value {
        Value::Array(cards.iter().map(|c| Value::String(self.card_ref(*c))).collect())
    }

    pub fn prompt_list(&self, r: ListRef) -> &[CardId] {
        match r {
            ListRef::Temp(i) => self.temps[i as usize].as_slice(),
            _ => self.st.list(r).as_slice(),
        }
    }

    /// Selectable flags for a ChooseCards prompt.
    pub fn choose_cards_selectable(&self, cards: ListRef, filter: &Filter, opts: &ChooseCardsOpts) -> Vec<bool> {
        self.prompt_list(cards)
            .iter()
            .enumerate()
            .map(|(i, c)| !opts.blocked.contains(&(i as u8)) && filter.matches(self.st.cdef(*c)))
            .collect()
    }

    pub fn choose_pokemon_candidates(&self, pr: &PromptRec) -> Vec<CardTarget> {
        if let PromptKind::ChoosePokemon { player_type, slots, blocked, .. } = pr.kind {
            let p = self.st.player_index_by_id(pr.perspective_id());
            let bl = blocked_slots(&self.st, p, blocked.as_slice());
            slot_targets(&self.st, p, player_type, slots.as_slice())
                .into_iter()
                .filter(|t| get_target(&self.st, p, *t).map(|s| !bl.contains(&s)).unwrap_or(false))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Canonical descriptor (`describePrompt` in the oracle).
    pub fn describe_prompt(&self, pr: &PromptRec) -> Value {
        let mut base = serde_json::Map::new();
        base.insert("kind".into(), json!("prompt"));
        base.insert("type".into(), json!(pr.type_name()));
        base.insert("cls".into(), json!(pr.class_name()));
        base.insert("player".into(), json!(self.st.player_index_by_id(pr.player_id)));
        if let Some(pp) = pr.perspective {
            base.insert("perspective".into(), json!(self.st.player_index_by_id(pp)));
        }
        if !pr.message.is_empty() {
            base.insert("message".into(), json!(pr.message));
        }
        let p = self.st.player_index_by_id(pr.perspective_id());
        match pr.kind {
            PromptKind::ChooseCards { cards, filter, opts } => {
                let list = self.prompt_list(cards);
                base.insert("cards".into(), self.refs(list));
                base.insert("selectable".into(), json!(self.choose_cards_selectable(cards, &filter, &opts)));
                base.insert("filter".into(), filter.to_json());
                let mut o = serde_json::Map::new();
                o.insert("min".into(), json!(opts.min));
                o.insert("max".into(), json!(opts.max));
                o.insert("allowCancel".into(), json!(opts.allow_cancel));
                o.insert("blocked".into(), json!(opts.blocked.as_slice()));
                o.insert("isSecret".into(), json!(opts.is_secret));
                o.insert("differentTypes".into(), json!(opts.different_types));
                o.insert("allowDifferentSuperTypes".into(), json!(opts.allow_different_super_types));
                for (k, v) in [
                    ("maxPokemons", opts.max_pokemons),
                    ("maxBasicEnergies", opts.max_basic_energies),
                    ("maxEnergies", opts.max_energies),
                    ("maxTrainers", opts.max_trainers),
                    ("maxTools", opts.max_tools),
                    ("maxStadiums", opts.max_stadiums),
                    ("maxSupporters", opts.max_supporters),
                    ("maxSpecialEnergies", opts.max_special_energies),
                    ("maxItems", opts.max_items),
                    ("maxBasics", opts.max_basics),
                    ("maxEvolutions", opts.max_evolutions),
                ] {
                    if let Some(v) = v {
                        o.insert(k.into(), json!(v));
                    }
                }
                base.insert("options".into(), Value::Object(o));
            }
            PromptKind::ChoosePokemon { player_type, slots, min, max, allow_cancel, blocked } => {
                base.insert("playerType".into(), json!(player_type as u8));
                base.insert("slots".into(), json!(slots.as_slice()));
                base.insert(
                    "options".into(),
                    json!({ "min": min, "max": max, "allowCancel": allow_cancel,
                            "blocked": blocked.iter().map(|t| target_json(*t)).collect::<Vec<_>>() }),
                );
                base.insert("candidates".into(), Value::Array(self.choose_pokemon_candidates(pr).into_iter().map(target_json).collect()));
            }
            PromptKind::ChoosePrize { count, blocked, use_opponent_prizes, allow_cancel, is_secret, .. } => {
                let q = if use_opponent_prizes { 1 - p } else { p };
                let left = self.st.players[q].prizes.iter().filter(|l| !l.is_empty()).count();
                base.insert("prizes".into(), json!(left));
                base.insert(
                    "options".into(),
                    json!({ "count": count, "max": count, "blocked": blocked.as_slice(), "allowCancel": allow_cancel,
                            "isSecret": is_secret, "useOpponentPrizes": use_opponent_prizes }),
                );
            }
            PromptKind::Select { values, allow_cancel, default_value } => {
                base.insert("values".into(), json!(values.values()));
                base.insert("options".into(), json!({ "allowCancel": allow_cancel, "defaultValue": default_value }));
            }
            PromptKind::ChooseEnergy { energy, cost, allow_cancel } => {
                base.insert(
                    "energy".into(),
                    Value::Array(
                        energy.iter().map(|e| json!({ "card": self.card_ref(e.card), "provides": e.provides.as_slice() })).collect(),
                    ),
                );
                base.insert("cost".into(), json!(cost.as_slice()));
                base.insert("options".into(), json!({ "allowCancel": allow_cancel }));
            }
            PromptKind::Confirm => {}
            _ => {
                base.insert("unknown".into(), json!(true));
            }
        }
        Value::Object(base)
    }

    // -----------------------------------------------------------------------
    // Decode + validate

    /// Decode a raw wire answer and validate it (socket-server semantics).
    pub fn decode_answer(&self, pr: &PromptRec, raw: &Value) -> Result<Res, GameError> {
        let invalid = GameError("INVALID_PROMPT_RESULT");
        let p = self.st.player_index_by_id(pr.perspective_id());
        if raw.is_null() {
            let can_cancel = match pr.kind {
                PromptKind::ChooseCards { opts, .. } => opts.allow_cancel,
                PromptKind::ChoosePokemon { allow_cancel, .. } => allow_cancel,
                PromptKind::ChoosePrize { allow_cancel, .. } => allow_cancel,
                PromptKind::ChooseEnergy { allow_cancel, .. } => allow_cancel,
                // Select / Confirm have no validate(): null is accepted.
                PromptKind::Select { .. } | PromptKind::Confirm => true,
                _ => true,
            };
            return if can_cancel { Ok(Res::Null) } else { Err(invalid) };
        }
        match pr.kind {
            PromptKind::Confirm => Ok(Res::Bool(raw.as_bool().ok_or(invalid)?)),
            PromptKind::Select { .. } => Ok(Res::Int(raw.as_i64().ok_or(invalid)? as i32)),
            PromptKind::ChooseCards { cards, filter, opts } => {
                let list = self.prompt_list(cards);
                let mut out: SVec<CardId, 16> = SVec::new();
                for v in raw.as_array().ok_or(invalid)? {
                    let i = v.as_u64().ok_or(invalid)? as usize;
                    // `cards[index]` is undefined for out-of-range indices; validate rejects it.
                    out.push(*list.get(i).ok_or(invalid)?);
                }
                if self.choose_cards_valid(list, out.as_slice(), &filter, &opts) {
                    Ok(Res::Cards(out))
                } else {
                    Err(invalid)
                }
            }
            PromptKind::ChoosePokemon { min, max, blocked, .. } => {
                let mut out: SVec<SlotRef, 8> = SVec::new();
                for v in raw.as_array().ok_or(invalid)? {
                    let t = target_from_json(v).ok_or(invalid)?;
                    let q = if t.player == PlayerType::BottomPlayer { p } else { 1 - p };
                    let s = if t.slot == SlotType::Active {
                        self.st.players[q].active
                    } else {
                        *self.st.players[q].bench.get(t.index as usize).ok_or(invalid)?
                    };
                    out.push(SlotRef::new(q, s));
                }
                if out.len() < min as usize || out.len() > max as usize {
                    return Err(invalid);
                }
                if out.iter().any(|s| self.st.slot(s.p as usize, s.s).cards.is_empty()) {
                    return Err(invalid);
                }
                let bl = blocked_slots(&self.st, p, blocked.as_slice());
                if out.iter().any(|s| bl.contains(s)) {
                    return Err(invalid);
                }
                Ok(Res::Slots(out))
            }
            PromptKind::ChoosePrize { count, use_opponent_prizes, .. } => {
                let q = if use_opponent_prizes { 1 - p } else { p };
                let nonempty: Vec<u8> =
                    (0..self.st.players[q].prize_count).filter(|i| !self.st.players[q].prizes[*i as usize].is_empty()).collect();
                let mut out: SVec<u8, 6> = SVec::new();
                for v in raw.as_array().ok_or(invalid)? {
                    let i = v.as_u64().ok_or(invalid)? as usize;
                    out.push(*nonempty.get(i).ok_or(invalid)?);
                }
                let required = (count as usize).min(nonempty.len());
                if out.len() != required {
                    return Err(invalid);
                }
                for (i, a) in out.iter().enumerate() {
                    if out.as_slice()[..i].contains(a) {
                        return Err(invalid);
                    }
                }
                Ok(Res::Prizes(out))
            }
            PromptKind::ChooseEnergy { energy, cost, .. } => {
                let mut out: SVec<EnergyEntry, 40> = SVec::new();
                for v in raw.as_array().ok_or(invalid)? {
                    let i = v.as_u64().ok_or(invalid)? as usize;
                    out.push(*energy.get(i).ok_or(invalid)?);
                }
                if energy::check_exact_energy(out.as_slice(), cost.as_slice()) {
                    Ok(Res::Energy(out))
                } else {
                    Err(invalid)
                }
            }
            PromptKind::ShuffleDeck => {
                let mut l: List<60> = List::new();
                for v in raw.as_array().ok_or(invalid)? {
                    l.push(v.as_u64().ok_or(invalid)? as u8);
                }
                Ok(Res::Order(l))
            }
            PromptKind::CoinFlip => Ok(Res::Bool(raw.as_bool().ok_or(invalid)?)),
            _ => Ok(Res::True),
        }
    }

    /// `chooseCardsSelectionValid`.
    pub fn choose_cards_valid(&self, cards: &[CardId], result: &[CardId], filter: &Filter, o: &ChooseCardsOpts) -> bool {
        if result.len() < o.min as usize || result.len() > o.max as usize {
            return false;
        }
        let defs: Vec<&CardDef> = result.iter().map(|c| self.st.cdef(*c)).collect();
        if !o.allow_different_super_types {
            if defs.iter().any(|d| d.super_type != defs[0].super_type) {
                return false;
            }
        }
        if o.different_types {
            let mut seen = Vec::new();
            for d in &defs {
                let t = choose_cards_card_type(d);
                if seen.contains(&t) {
                    return false;
                }
                seen.push(t);
            }
        }
        let count = |f: &dyn Fn(&CardDef) -> bool| defs.iter().filter(|d| f(d)).count();
        let pokemon = count(&|d| d.is_pokemon());
        let basics = count(&|d| d.is_pokemon() && d.stage == Stage::Basic as u8);
        if (o.max_basics.is_some() || o.max_evolutions.is_some()) && basics > 0 && pokemon - basics > 0 {
            return false;
        }
        let over = |m: Option<u8>, n: usize| m.map(|m| (m as usize) < n).unwrap_or(false);
        if over(o.max_pokemons, pokemon)
            || over(o.max_basic_energies, count(&|d| d.is_energy() && d.energy_type == 0))
            || over(o.max_energies, count(&|d| d.is_energy()))
            || over(o.max_trainers, count(&|d| d.is_trainer()))
            || over(o.max_items, count(&|d| d.is_trainer() && d.trainer_type == 0))
            || over(o.max_stadiums, count(&|d| d.is_trainer() && d.trainer_type == 2))
            || over(o.max_supporters, count(&|d| d.is_trainer() && d.trainer_type == 1))
            || over(o.max_special_energies, count(&|d| d.is_energy() && d.energy_type == 1))
            || over(o.max_tools, count(&|d| d.is_trainer() && d.trainer_type == 3))
            || over(o.max_basics, basics)
            || over(o.max_evolutions, pokemon - basics)
        {
            return false;
        }
        result.iter().all(|c| match cards.iter().position(|x| x == c) {
            Some(i) => !o.blocked.contains(&(i as u8)) && filter.matches(self.st.cdef(*c)),
            None => false,
        })
    }
}

/// `ChooseCardsPrompt.getCardType`.
fn choose_cards_card_type(d: &CardDef) -> CardType {
    if d.is_energy() {
        return d.provides.first().copied().unwrap_or(ct::NONE);
    }
    if d.is_pokemon() {
        return d.card_type.first().copied().unwrap_or(ct::COLORLESS);
    }
    ct::NONE
}
