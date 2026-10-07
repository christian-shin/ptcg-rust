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

/// Set of small indices (e.g. blocked card positions), in ascending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Blocked(pub u64);

impl Blocked {
    pub fn push(&mut self, i: u8) {
        self.0 |= 1u64 << i;
    }
    pub fn contains(&self, i: &u8) -> bool {
        *i < 64 && self.0 & (1u64 << *i) != 0
    }
    pub fn to_vec(&self) -> Vec<u8> {
        (0..64).filter(|i| self.0 & (1u64 << i) != 0).collect()
    }
}

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
    /// `evolvesFrom` (compared with `!==`; non-Pokémon never match).
    pub evolves_from: Option<&'static str>,
    /// `cardType` was given as a one-element array (`cardType: [T]`).
    pub card_type_list: bool,
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
        if let Some(n) = self.evolves_from {
            if !d.is_pokemon() || d.evolves_from != n {
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
            if self.card_type_list {
                m.insert("cardType".into(), json!([v]));
            } else {
                m.insert("cardType".into(), json!(v));
            }
        }
        if let Some(v) = self.tags {
            m.insert("tags".into(), json!([TAG_NAMES[v as usize]]));
        }
        if let Some(v) = self.name {
            m.insert("name".into(), json!(crate::carddb::tl_card_name(v)));
        }
        if let Some(v) = self.evolves_from {
            m.insert("evolvesFrom".into(), json!(crate::carddb::tl_card_name(v)));
        }
        Value::Object(m)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChooseCardsOpts {
    pub min: u8,
    pub max: u8,
    pub allow_cancel: bool,
    pub blocked: Blocked,
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
    pub max_stage1: Option<u8>,
    pub max_stage2: Option<u8>,
}

impl ChooseCardsOpts {
    pub fn new(min: u8, max: u8, allow_cancel: bool) -> Self {
        ChooseCardsOpts {
            min,
            max,
            allow_cancel,
            blocked: Blocked::default(),
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
            max_stage1: None,
            max_stage2: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SelectValues {
    /// "Draw N card(s)" for N from `max` down to 0 (mulligan / DRAW_UP_TO_X_CARDS).
    DrawCards(u8),
    Static(&'static [&'static str]),
    /// Card names chosen at run time (the order of step 7 triggers).
    Dyn(SVec<&'static str, 8>),
}

impl SelectValues {
    pub fn len(&self) -> usize {
        match self {
            SelectValues::DrawCards(m) => *m as usize + 1,
            SelectValues::Static(v) => v.len(),
            SelectValues::Dyn(v) => v.len(),
        }
    }
    pub fn values(&self) -> Vec<String> {
        match self {
            SelectValues::DrawCards(m) => (0..=*m).rev().map(|i| format!("Draw {} card(s)", i)).collect(),
            SelectValues::Static(v) => v.iter().map(|s| s.to_string()).collect(),
            SelectValues::Dyn(v) => v.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Blocked targets of a ChoosePokemonPrompt (both sides, bench up to 8).
pub type TargetList = SVec<CardTarget, 18>;

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
    ChoosePokemon { player_type: PlayerType, slots: SVec<u8, 3>, min: u8, max: u8, allow_cancel: bool, blocked: TargetList },
    ChoosePrize { count: u8, blocked: SVec<u8, 6>, use_opponent_prizes: bool, allow_cancel: bool, is_secret: bool, destination: Option<ListRef>, face_down_only: bool },
    Select { values: SelectValues, allow_cancel: bool, default_value: i32 },
    ChooseEnergy { energy: EnergyMap, cost: Cost, allow_cancel: bool },
    AttachEnergy { cards: ListRef, player_type: PlayerType, slots: SVec<u8, 3>, filter: Filter, o: AttachOpts },
    DiscardEnergy { player_type: PlayerType, slots: SVec<u8, 3>, filter: Filter, o: MoveOpts },
    MoveEnergy { player_type: PlayerType, slots: SVec<u8, 3>, filter: Filter, o: MoveOpts },
    PutDamage { player_type: PlayerType, slots: SVec<u8, 3>, damage: i32, max_allowed: SVec<(CardTarget, i32), 16>, allow_cancel: bool, blocked: SVec<CardTarget, 16>, allow_partial: bool, damage_multiple: i32 },
    MoveDamage { player_type: PlayerType, slots: SVec<u8, 3>, max_allowed: SVec<(CardTarget, i32), 16>, o: MoveOpts, single_source: bool, single_destination: bool, damage_multiple: i32 },
    RemoveDamage { player_type: PlayerType, slots: SVec<u8, 3>, max_allowed: SVec<(CardTarget, i32), 16>, o: MoveOpts, same_target: bool },
    OrderCards { cards: ListRef, allow_cancel: bool },
    SelectOption { values: &'static [&'static str], allow_cancel: bool, default_value: i32, disabled: Option<u16> },
    ChooseAttack { cards: SVec<CardId, 64>, allow_cancel: bool, blocked_message: &'static str, blocked: SVec<(u8, u8), 16> },
}

#[derive(Clone, Copy, Debug)]
pub struct AttachOpts {
    pub allow_cancel: bool,
    pub min: u8,
    pub max: u8,
    pub blocked: Blocked,
    pub blocked_to: SVec<CardTarget, 9>,
    pub different_types: bool,
    pub same_target: bool,
    pub different_targets: bool,
    pub valid_card_types: Option<SVec<CardType, 4>>,
    pub max_per_type: Option<u8>,
}

impl AttachOpts {
    /// Defaults with `max` = number of cards in the source list.
    pub fn new(max: u8) -> Self {
        AttachOpts {
            allow_cancel: true,
            min: 0,
            max,
            blocked: Blocked::default(),
            blocked_to: SVec::new(),
            different_types: false,
            same_target: false,
            different_targets: false,
            valid_card_types: None,
            max_per_type: None,
        }
    }
}

/// Options shared by Discard/Move energy and Move/Remove damage prompts.
#[derive(Clone, Copy, Debug)]
pub struct MoveOpts {
    pub allow_cancel: bool,
    pub min: u8,
    pub max: Option<u8>,
    // Up to 18 entries: every Pokémon in play on both sides (2 x 9 slots).
    pub blocked_from: SVec<CardTarget, 18>,
    pub blocked_to: SVec<CardTarget, 18>,
    /// (source, blocked card indices).
    pub blocked_map: SVec<(CardTarget, Blocked), 18>,
}

impl Default for MoveOpts {
    fn default() -> Self {
        MoveOpts { allow_cancel: true, min: 0, max: None, blocked_from: SVec::new(), blocked_to: SVec::new(), blocked_map: SVec::new() }
    }
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
    /// Trainer whose resolution created this prompt (not serialized).
    pub trainer: Option<(u8, CardId)>,
}

/// Decoded prompt result (what Twinleaf stores in `prompt.result`).
#[derive(Clone, Copy, Debug)]
pub enum Res {
    Null,
    True,
    Bool(bool),
    Int(i32),
    Cards(List<120>),
    Slots(SVec<SlotRef, 8>),
    /// Chosen energy entries, by card.
    Energy(SVec<CardId, 64>),
    /// Indices into the owning player's `prizes` array.
    Prizes(SVec<u8, 6>),
    Order(List<120>),
    /// AttachEnergyPrompt: (to, card).
    Attach(SVec<(CardTarget, CardId), 64>),
    /// DiscardEnergyPrompt: (from, card).
    CardsFrom(SVec<(CardTarget, CardId), 64>),
    /// MoveEnergyPrompt: (from, to, card).
    Transfers(SVec<(CardTarget, CardTarget, CardId), 64>),
    /// PutDamagePrompt: (target, damage).
    DamageMap(SVec<(CardTarget, i32), 16>),
    /// Move/RemoveDamagePrompt: (from, to, count). One entry per run of identical
    /// consecutive transfers: Twinleaf's resolver answers with one transfer per
    /// damage counter (32 and more in one answer), so the list is run-length
    /// encoded; [`damage_transfers`] expands it back to one pair per transfer.
    DamageTransfers(SVec<(CardTarget, CardTarget, u8), MAX_DAMAGE_RUNS>),
    Attack(AttackRef),
}

/// Runs of a decoded Move/RemoveDamagePrompt answer (the bot's answers have at
/// most one run per from/to Pokémon, 17 at the most; the oracle's random
/// policy and the agent interface answer with at most 12 transfers).
pub const MAX_DAMAGE_RUNS: usize = 32;

/// The transfers of a [`Res::DamageTransfers`] in answer order, one (from, to)
/// pair per transfer (`for (const transfer of transfers)`).
pub fn damage_transfers(runs: &[(CardTarget, CardTarget, u8)]) -> impl Iterator<Item = (CardTarget, CardTarget)> + '_ {
    runs.iter().flat_map(|(f, t, n)| std::iter::repeat((*f, *t)).take(*n as usize))
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
            PromptKind::AttachEnergy { .. } => "Attach energy",
            PromptKind::DiscardEnergy { .. } => "Discard energy",
            PromptKind::MoveEnergy { .. } => "Move energy",
            PromptKind::PutDamage { .. } => "Put damage",
            PromptKind::MoveDamage { .. } => "Move damage",
            PromptKind::RemoveDamage { .. } => "Remove damage",
            PromptKind::OrderCards { .. } => "Order cards",
            PromptKind::SelectOption { .. } => "SelectOption",
            PromptKind::ChooseAttack { .. } => "Choose attack",
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
            PromptKind::AttachEnergy { .. } => "AttachEnergyPrompt",
            PromptKind::DiscardEnergy { .. } => "DiscardEnergyPrompt",
            PromptKind::MoveEnergy { .. } => "MoveEnergyPrompt",
            PromptKind::PutDamage { .. } => "PutDamagePrompt",
            PromptKind::MoveDamage { .. } => "MoveDamagePrompt",
            PromptKind::RemoveDamage { .. } => "RemoveDamagePrompt",
            PromptKind::OrderCards { .. } => "OrderCardsPrompt",
            PromptKind::SelectOption { .. } => "SelectOptionPrompt",
            PromptKind::ChooseAttack { .. } => "ChooseAttackPrompt",
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
        format!("{}-{}#{}", d.tl_set, d.tl_set_number, c)
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
                o.insert("blocked".into(), json!(opts.blocked.to_vec()));
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
                    ("maxStage1", opts.max_stage1),
                    ("maxStage2", opts.max_stage2),
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
            PromptKind::ChoosePrize { count, blocked, use_opponent_prizes, allow_cancel, is_secret, face_down_only, .. } => {
                let q = if use_opponent_prizes { 1 - p } else { p };
                let left = self.st.players[q].prizes.iter().filter(|l| !l.is_empty()).count();
                base.insert("prizes".into(), json!(left));
                let mut opts = json!({ "count": count, "max": count, "blocked": blocked.as_slice(), "allowCancel": allow_cancel,
                            "isSecret": is_secret, "useOpponentPrizes": use_opponent_prizes });
                if face_down_only {
                    opts["faceDownOnly"] = json!(true);
                }
                base.insert("options".into(), opts);
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
                if !self.describe_extra(pr, &mut base) {
                    base.insert("unknown".into(), json!(true));
                }
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
                PromptKind::AttachEnergy { o, .. } => o.allow_cancel,
                PromptKind::DiscardEnergy { o, .. } | PromptKind::MoveEnergy { o, .. } => o.allow_cancel,
                PromptKind::MoveDamage { o, .. } | PromptKind::RemoveDamage { o, .. } => o.allow_cancel,
                PromptKind::PutDamage { allow_cancel, .. } => allow_cancel,
                PromptKind::OrderCards { allow_cancel, .. } => allow_cancel,
                PromptKind::ChooseAttack { allow_cancel, .. } => allow_cancel,
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
                let mut out: List<120> = List::new();
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
            PromptKind::ChoosePrize { count, use_opponent_prizes, face_down_only, .. } => {
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
                if face_down_only && out.iter().any(|a| self.st.players[q].prize_face_up[*a as usize]) {
                    return Err(invalid);
                }
                Ok(Res::Prizes(out))
            }
            PromptKind::ChooseEnergy { energy, cost, .. } => {
                let mut out: SVec<EnergyEntry, 64> = SVec::new();
                for v in raw.as_array().ok_or(invalid)? {
                    let i = v.as_u64().ok_or(invalid)? as usize;
                    out.push(*energy.get(i).ok_or(invalid)?);
                }
                if energy::check_energy_payment(out.as_slice(), cost.as_slice()) {
                    let mut cards: SVec<CardId, 64> = SVec::new();
                    for e in out.iter() {
                        cards.push(e.card);
                    }
                    Ok(Res::Energy(cards))
                } else {
                    Err(invalid)
                }
            }
            PromptKind::ShuffleDeck => {
                let mut l: List<120> = List::new();
                for v in raw.as_array().ok_or(invalid)? {
                    l.push(v.as_u64().ok_or(invalid)? as u8);
                }
                Ok(Res::Order(l))
            }
            PromptKind::CoinFlip => Ok(Res::Bool(raw.as_bool().ok_or(invalid)?)),
            PromptKind::SelectOption { .. } => Ok(Res::Int(raw.as_i64().ok_or(invalid)? as i32)),
            PromptKind::ChooseAttack { cards, blocked, .. } => {
                let i = raw.get("index").and_then(|x| x.as_i64()).ok_or(invalid)?;
                let name = raw.get("attack").and_then(|x| x.as_str()).ok_or(invalid)?;
                if i < 0 || i as usize >= cards.len() {
                    return Err(invalid);
                }
                let c = *cards.get(i as usize).unwrap();
                let ai = self.st.cdef(c).attacks.iter().position(|a| crate::carddb::attack_is(a, name)).ok_or(invalid)?;
                // validate: blocked attacks are rejected (matched by object identity).
                if blocked.iter().any(|(bi, ba)| *bi as usize == i as usize && *ba as usize == ai) {
                    return Err(invalid);
                }
                Ok(Res::Attack(AttackRef { card: c, index: ai as u8 }))
            }
            PromptKind::AttachEnergy { .. }
            | PromptKind::DiscardEnergy { .. }
            | PromptKind::MoveEnergy { .. }
            | PromptKind::PutDamage { .. }
            | PromptKind::MoveDamage { .. }
            | PromptKind::RemoveDamage { .. }
            | PromptKind::OrderCards { .. } => self.decode_extra(pr, raw).unwrap_or(Err(invalid)),
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
        if (o.max_basics.is_some() || o.max_evolutions.is_some())
            && o.max_stage1.is_none()
            && o.max_stage2.is_none()
            && basics > 0
            && pokemon - basics > 0
        {
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
            || over(o.max_stage1, count(&|d| d.is_pokemon() && d.stage == Stage::Stage1 as u8))
            || over(o.max_stage2, count(&|d| d.is_pokemon() && d.stage == Stage::Stage2 as u8))
        {
            return false;
        }
        result.iter().all(|c| match cards.iter().position(|x| x == c) {
            Some(i) => !o.blocked.contains(&(i as u8)) && filter.matches(self.st.cdef(*c)),
            None => false,
        })
    }
}

/// A discard prompt whose filter asks for Pokémon Tools chooses attached Tools (across Pokémon)
/// instead of Energy (`isToolFilter` in discard-energy-prompt.ts).
pub fn is_tool_filter(f: &Filter) -> bool {
    f.super_type == Some(SuperType::Trainer as u8) && f.trainer_type == Some(TrainerType::Tool as u8)
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

// ---------------------------------------------------------------------------
// Energy / damage prompts

fn targets_json(ts: &[CardTarget]) -> Value {
    Value::Array(ts.iter().map(|t| target_json(*t)).collect())
}

fn damage_map_json(m: &[(CardTarget, i32)]) -> Value {
    Value::Array(m.iter().map(|(t, d)| json!({ "target": target_json(*t), "damage": d })).collect())
}

fn move_opts_json(o: &MoveOpts, with_blocked_to: bool) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    m.insert("allowCancel".into(), json!(o.allow_cancel));
    m.insert("min".into(), json!(o.min));
    if let Some(x) = o.max {
        m.insert("max".into(), json!(x));
    }
    m.insert("blockedFrom".into(), targets_json(o.blocked_from.as_slice()));
    if with_blocked_to {
        m.insert("blockedTo".into(), targets_json(o.blocked_to.as_slice()));
    }
    m
}

fn blocked_map_json(o: &MoveOpts) -> Value {
    Value::Array(o.blocked_map.iter().map(|(s, b)| json!({ "source": target_json(*s), "blocked": b.to_vec() })).collect())
}

fn same_t(a: CardTarget, b: CardTarget) -> bool {
    a.player == b.player && a.slot == b.slot && a.index == b.index
}

fn in_targets(list: &[CardTarget], t: CardTarget) -> bool {
    list.iter().any(|x| same_t(*x, t))
}

fn target_from(v: &Value, key: &str) -> Option<CardTarget> {
    target_from_json(v.get(key)?)
}

impl Game {
    /// `energySources` in the oracle: slots and energy card indices offered.
    pub fn energy_sources(&self, p: usize, pt: PlayerType, slots: &[u8], filter: &Filter, o: &MoveOpts) -> Vec<(CardTarget, Vec<u8>)> {
        let mut out = Vec::new();
        for from in slot_targets(&self.st, p, pt, slots) {
            if in_targets(o.blocked_from.as_slice(), from) {
                continue;
            }
            let s = get_target(&self.st, p, from).unwrap();
            let blocked = o.blocked_map.iter().find(|(src, _)| same_t(*src, from)).map(|x| x.1).unwrap_or_default();
            // A Tool filter chooses from the Pokémon's `tools` (not in `cards`); `index` is its position there.
            let tool_mode = is_tool_filter(filter);
            let slot = self.st.slot(s.p as usize, s.s);
            let list: &[CardId] = if tool_mode { slot.tools.as_slice() } else { slot.cards.as_slice() };
            let idx: Vec<u8> = list
                .iter()
                .copied()
                .enumerate()
                .filter(|(i, c)| {
                    let d = self.st.cdef(*c);
                    (tool_mode || d.is_energy()) && !blocked.contains(&(*i as u8)) && filter.matches(d)
                })
                .map(|(i, _)| i as u8)
                .collect();
            if !idx.is_empty() {
                out.push((from, idx));
            }
        }
        out
    }

    /// Descriptor fields for the energy / damage prompt kinds.
    pub fn describe_extra(&self, pr: &PromptRec, base: &mut serde_json::Map<String, Value>) -> bool {
        let p = self.st.player_index_by_id(pr.perspective_id());
        match pr.kind {
            PromptKind::AttachEnergy { cards, player_type, slots, filter, o } => {
                let list = self.prompt_list(cards);
                base.insert("cards".into(), Value::Array(list.iter().map(|c| json!(self.card_ref(*c))).collect()));
                base.insert(
                    "selectable".into(),
                    json!(list
                        .iter()
                        .enumerate()
                        .map(|(i, c)| {
                            let d = self.st.cdef(*c);
                            d.is_energy() && !o.blocked.contains(&(i as u8)) && filter.matches(d)
                        })
                        .collect::<Vec<_>>()),
                );
                base.insert("playerType".into(), json!(player_type as u8));
                base.insert("slots".into(), json!(slots.as_slice()));
                base.insert("filter".into(), filter.to_json());
                let mut m = serde_json::Map::new();
                m.insert("allowCancel".into(), json!(o.allow_cancel));
                m.insert("min".into(), json!(o.min));
                m.insert("max".into(), json!(o.max));
                m.insert("blocked".into(), json!(o.blocked.to_vec()));
                m.insert("blockedTo".into(), targets_json(o.blocked_to.as_slice()));
                m.insert("differentTypes".into(), json!(o.different_types));
                m.insert("sameTarget".into(), json!(o.same_target));
                m.insert("differentTargets".into(), json!(o.different_targets));
                if let Some(v) = o.valid_card_types {
                    m.insert("validCardTypes".into(), json!(v.as_slice()));
                }
                if let Some(v) = o.max_per_type {
                    m.insert("maxPerType".into(), json!(v));
                }
                base.insert("options".into(), Value::Object(m));
                let targets: Vec<CardTarget> =
                    slot_targets(&self.st, p, player_type, slots.as_slice()).into_iter().filter(|t| !in_targets(o.blocked_to.as_slice(), *t)).collect();
                base.insert("targets".into(), targets_json(&targets));
            }
            PromptKind::DiscardEnergy { player_type, slots, filter, o } | PromptKind::MoveEnergy { player_type, slots, filter, o } => {
                let is_move = matches!(pr.kind, PromptKind::MoveEnergy { .. });
                base.insert("playerType".into(), json!(player_type as u8));
                base.insert("slots".into(), json!(slots.as_slice()));
                base.insert("filter".into(), filter.to_json());
                let mut m = move_opts_json(&o, is_move);
                m.insert("blockedMap".into(), blocked_map_json(&o));
                base.insert("options".into(), Value::Object(m));
                let src = self.energy_sources(p, player_type, slots.as_slice(), &filter, &o);
                base.insert("sources".into(), Value::Array(src.iter().map(|(f, i)| json!({ "from": target_json(*f), "indices": i })).collect()));
            }
            PromptKind::PutDamage { player_type, slots, damage, max_allowed, allow_cancel, blocked, allow_partial, damage_multiple } => {
                base.insert("playerType".into(), json!(player_type as u8));
                base.insert("slots".into(), json!(slots.as_slice()));
                base.insert("damage".into(), json!(damage));
                base.insert("maxAllowedDamage".into(), damage_map_json(max_allowed.as_slice()));
                base.insert(
                    "options".into(),
                    json!({ "allowCancel": allow_cancel, "blocked": targets_json(blocked.as_slice()),
                            "allowPlacePartialDamage": allow_partial, "damageMultiple": damage_multiple }),
                );
            }
            PromptKind::MoveDamage { player_type, slots, max_allowed, o, single_source, single_destination, damage_multiple } => {
                base.insert("playerType".into(), json!(player_type as u8));
                base.insert("slots".into(), json!(slots.as_slice()));
                base.insert("maxAllowedDamage".into(), damage_map_json(max_allowed.as_slice()));
                let mut m = move_opts_json(&o, true);
                m.insert("singleSourceTarget".into(), json!(single_source));
                m.insert("singleDestinationTarget".into(), json!(single_destination));
                m.insert("damageMultiple".into(), json!(damage_multiple));
                base.insert("options".into(), Value::Object(m));
            }
            PromptKind::RemoveDamage { player_type, slots, max_allowed, o, same_target } => {
                base.insert("playerType".into(), json!(player_type as u8));
                base.insert("slots".into(), json!(slots.as_slice()));
                base.insert("maxAllowedDamage".into(), damage_map_json(max_allowed.as_slice()));
                let mut m = move_opts_json(&o, true);
                m.insert("sameTarget".into(), json!(same_target));
                base.insert("options".into(), Value::Object(m));
            }
            PromptKind::OrderCards { cards, allow_cancel } => {
                let list = self.prompt_list(cards);
                base.insert("cards".into(), Value::Array(list.iter().map(|c| json!(self.card_ref(*c))).collect()));
                base.insert("options".into(), json!({ "allowCancel": allow_cancel }));
            }
            PromptKind::SelectOption { values, allow_cancel, default_value, disabled } => {
                base.insert("values".into(), json!(values));
                let mut m = serde_json::Map::new();
                m.insert("allowCancel".into(), json!(allow_cancel));
                m.insert("defaultValue".into(), json!(default_value));
                if let Some(d) = disabled {
                    m.insert("disabled".into(), json!((0..values.len()).map(|i| d & (1 << i) != 0).collect::<Vec<_>>()));
                }
                base.insert("options".into(), Value::Object(m));
            }
            PromptKind::ChooseAttack { cards, allow_cancel, blocked_message, blocked } => {
                base.insert("cards".into(), Value::Array(cards.iter().map(|c| json!(self.card_ref(*c))).collect()));
                base.insert(
                    "attacks".into(),
                    Value::Array(cards.iter().map(|c| json!(self.st.cdef(*c).attacks.iter().map(|a| a.tl_name).collect::<Vec<_>>())).collect()),
                );
                base.insert(
                    "options".into(),
                    json!({ "allowCancel": allow_cancel, "blockedMessage": blocked_message,
                            "blocked": blocked.iter().map(|(i, a)| json!({ "index": i, "attack": self.st.cdef(*cards.get(*i as usize).unwrap()).attacks[*a as usize].tl_name })).collect::<Vec<_>>() }),
                );
            }
            _ => return false,
        }
        true
    }

    /// Decode + validate for the energy / damage prompt kinds.
    pub fn decode_extra(&self, pr: &PromptRec, raw: &Value) -> Option<Result<Res, GameError>> {
        let invalid = GameError("INVALID_PROMPT_RESULT");
        let p = self.st.player_index_by_id(pr.perspective_id());
        let arr = match raw.as_array() {
            Some(a) => a,
            None => return None,
        };
        // Answers longer than the result capacity are rejected, not a panic
        // (only reachable from agents; no oracle answer comes close, except the
        // bot's Move damage answers, which are run-length encoded).
        let cap = match pr.kind {
            PromptKind::MoveEnergy { .. } => 48,
            PromptKind::OrderCards { .. } => 120,
            // Run-length encoded below: only the number of runs is limited.
            PromptKind::MoveDamage { .. } | PromptKind::RemoveDamage { .. } => usize::MAX,
            _ => 16,
        };
        if arr.len() > cap {
            return Some(Err(invalid));
        }
        let r = (|| -> Result<Res, GameError> {
            match pr.kind {
                PromptKind::AttachEnergy { cards, o, .. } => {
                    let list = self.prompt_list(cards);
                    let mut out: SVec<(CardTarget, CardId), 64> = SVec::new();
                    for v in arr {
                        let to = target_from(v, "to").ok_or(invalid)?;
                        let i = v.get("index").and_then(|x| x.as_u64()).ok_or(invalid)? as usize;
                        let c = *list.get(i).ok_or(invalid)?;
                        if !self.st.cdef(c).is_energy() || o.blocked.contains(&(i as u8)) {
                            return Err(invalid);
                        }
                        out.push((to, c));
                    }
                    let n = out.len();
                    if n < o.min as usize || n > o.max as usize {
                        return Err(invalid);
                    }
                    if out.iter().any(|(_, c)| list.iter().position(|x| x == c).map(|i| o.blocked.contains(&(i as u8))).unwrap_or(false)) {
                        return Err(invalid);
                    }
                    if let Some(m) = o.max_per_type {
                        let mut counts = [0u8; 32];
                        for (_, c) in out.iter() {
                            let t = self.st.cdef(*c).provides.first().copied().unwrap_or(0) as usize;
                            counts[t] += 1;
                            if counts[t] > m {
                                return Err(invalid);
                            }
                        }
                    }
                    if o.same_target && n > 1 {
                        let t = out.as_slice()[0].0;
                        if out.iter().any(|(x, _)| !same_t(*x, t)) {
                            return Err(invalid);
                        }
                    }
                    if let Some(v) = o.valid_card_types {
                        let ok = out.iter().all(|(_, c)| self.st.cdef(*c).provides.iter().any(|t| v.contains(t)));
                        if !ok {
                            return Err(invalid);
                        }
                    }
                    if o.different_types {
                        let mut seen = Vec::new();
                        for (_, c) in out.iter() {
                            let t = self.st.cdef(*c).provides.first().copied().unwrap_or(ct::NONE);
                            if seen.contains(&t) {
                                return Err(invalid);
                            }
                            seen.push(t);
                        }
                    }
                    if o.different_targets && n > 1 {
                        for (i, (t, _)) in out.iter().enumerate() {
                            if out.iter().position(|(x, _)| same_t(*x, *t)) != Some(i) {
                                return Err(invalid);
                            }
                        }
                    }
                    Ok(Res::Attach(out))
                }
                PromptKind::DiscardEnergy { o, filter, .. } => {
                    let mut out: SVec<(CardTarget, CardId), 64> = SVec::new();
                    let mut keys: Vec<(u8, u8, u8, usize)> = Vec::new();
                    for v in arr {
                        let from = target_from(v, "from").ok_or(invalid)?;
                        let i = v.get("index").and_then(|x| x.as_u64()).ok_or(invalid)? as usize;
                        let k = (from.player as u8, from.slot as u8, from.index, i);
                        if keys.contains(&k) {
                            return Err(invalid);
                        }
                        keys.push(k);
                        let s = get_target(&self.st, p, from)?;
                        let tool_mode = is_tool_filter(&filter);
                        let slot = self.st.slot(s.p as usize, s.s);
                        let c = *(if tool_mode { slot.tools.as_slice() } else { slot.cards.as_slice() }).get(i).ok_or(invalid)?;
                        if !tool_mode && !self.st.cdef(c).is_energy() {
                            return Err(invalid);
                        }
                        out.push((from, c));
                    }
                    if out.len() < o.min as usize || o.max.map(|m| out.len() > m as usize).unwrap_or(false) {
                        return Err(invalid);
                    }
                    Ok(Res::CardsFrom(out))
                }
                PromptKind::MoveEnergy { .. } => {
                    let mut out: SVec<(CardTarget, CardTarget, CardId), 64> = SVec::new();
                    for v in arr {
                        let from = target_from(v, "from").ok_or(invalid)?;
                        let to = target_from(v, "to").ok_or(invalid)?;
                        let i = v.get("index").and_then(|x| x.as_u64()).ok_or(invalid)? as usize;
                        let s = get_target(&self.st, p, from)?;
                        let c = *self.st.slot(s.p as usize, s.s).cards.as_slice().get(i).ok_or(invalid)?;
                        out.push((from, to, c));
                    }
                    Ok(Res::Transfers(out))
                }
                PromptKind::PutDamage { player_type, slots, damage, blocked, allow_partial, .. } => {
                    let mut out: SVec<(CardTarget, i32), 16> = SVec::new();
                    let mut sum = 0;
                    for v in arr {
                        let t = target_from(v, "target").ok_or(invalid)?;
                        let d = v.get("damage").and_then(|x| x.as_i64()).ok_or(invalid)? as i32;
                        sum += d;
                        out.push((t, d));
                    }
                    if sum != damage && !allow_partial {
                        return Err(invalid);
                    }
                    let bl = blocked_slots(&self.st, p, blocked.as_slice());
                    for (t, _) in out.iter() {
                        let s = get_target(&self.st, p, *t).map_err(|_| invalid)?;
                        if bl.contains(&s) {
                            return Err(invalid);
                        }
                    }
                    if player_type != PlayerType::Any && out.iter().any(|(t, _)| t.player != player_type) {
                        return Err(invalid);
                    }
                    if out.iter().any(|(t, _)| !slots.contains(&(t.slot as u8))) {
                        return Err(invalid);
                    }
                    Ok(Res::DamageMap(out))
                }
                PromptKind::MoveDamage { .. } | PromptKind::RemoveDamage { .. } => {
                    let (player_type, slots, o, single_source, single_destination) = match pr.kind {
                        PromptKind::MoveDamage { player_type, slots, o, single_source, single_destination, .. } => {
                            (player_type, slots, o, single_source, single_destination)
                        }
                        PromptKind::RemoveDamage { player_type, slots, o, same_target, .. } => (player_type, slots, o, false, same_target),
                        _ => unreachable!(),
                    };
                    // Run-length encoded (see `Res::DamageTransfers`); `n` counts the transfers.
                    let mut out: SVec<(CardTarget, CardTarget, u8), MAX_DAMAGE_RUNS> = SVec::new();
                    let mut n = 0usize;
                    for v in arr {
                        let (f, t) = (target_from(v, "from").ok_or(invalid)?, target_from(v, "to").ok_or(invalid)?);
                        n += 1;
                        let extend = matches!(out.as_slice().last(), Some(l) if same_t(l.0, f) && same_t(l.1, t) && l.2 < u8::MAX);
                        if extend {
                            out.as_mut_slice().last_mut().unwrap().2 += 1;
                        } else if out.len() == MAX_DAMAGE_RUNS {
                            return Err(invalid);
                        } else {
                            out.push((f, t, 1));
                        }
                    }
                    if single_source && n > 1 && out.iter().any(|(f, _, _)| !same_t(*f, out.as_slice()[0].0)) {
                        return Err(invalid);
                    }
                    if single_destination && n > 1 && out.iter().any(|(_, t, _)| !same_t(*t, out.as_slice()[0].1)) {
                        return Err(invalid);
                    }
                    if n < o.min as usize || o.max.map(|m| n > m as usize).unwrap_or(false) {
                        return Err(invalid);
                    }
                    let bf = blocked_slots(&self.st, p, o.blocked_from.as_slice());
                    let bt = blocked_slots(&self.st, p, o.blocked_to.as_slice());
                    for (f, t, _) in out.iter() {
                        let fs = get_target(&self.st, p, *f).map_err(|_| invalid)?;
                        let ts = get_target(&self.st, p, *t).map_err(|_| invalid)?;
                        if bf.contains(&fs) || bt.contains(&ts) {
                            return Err(invalid);
                        }
                    }
                    if player_type != PlayerType::Any && out.iter().any(|(f, t, _)| f.player != player_type || t.player != player_type) {
                        return Err(invalid);
                    }
                    if out.iter().any(|(f, t, _)| !slots.contains(&(f.slot as u8)) || !slots.contains(&(t.slot as u8))) {
                        return Err(invalid);
                    }
                    Ok(Res::DamageTransfers(out))
                }
                PromptKind::OrderCards { cards, .. } => {
                    let n = self.prompt_list(cards).len();
                    let mut v: Vec<u64> = Vec::new();
                    for x in arr {
                        v.push(x.as_u64().ok_or(invalid)?);
                    }
                    if v.len() != n {
                        return Err(invalid);
                    }
                    // `s.sort()` sorts numbers as strings.
                    let mut s = v.clone();
                    s.sort_by_key(|x| x.to_string());
                    if s.iter().enumerate().any(|(i, x)| *x != i as u64) {
                        return Err(invalid);
                    }
                    Ok(Res::Order(List::from_slice(&v.iter().map(|x| *x as u8).collect::<Vec<_>>())))
                }
                _ => Err(invalid),
            }
        })();
        Some(r)
    }
}
