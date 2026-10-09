//! Engine-driven selection interface in the shape of the Kaggle cabt engine:
//! every decision is a [`SelectData`] with `type`, `context`, `min_count`,
//! `max_count` and an explicit `option` list; the agent answers with option
//! indices.
//!
//! Answers are translated to Twinleaf wire answers and go through the same
//! decode + validate path as oracle traces, so the interface cannot accept
//! anything the rules engine would reject.

use crate::game::{Action, Game, GameError, Pending, R};
use crate::options::TurnOption;
use crate::prompts::*;
use crate::state::ListRef;
use crate::types::*;
use crate::list::CardList;
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SelectType {
    Main = 0,
    Card = 1,
    AttachedCard = 2,
    CardOrAttachedCard = 3,
    Energy = 4,
    Skill = 5,
    Attack = 6,
    Evolve = 7,
    Count = 8,
    YesNo = 9,
    SpecialCondition = 10,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SelectContext {
    Main = 0,
    SetupActivePokemon = 1,
    SetupBenchPokemon = 2,
    Switch = 3,
    ToActive = 4,
    ToBench = 5,
    ToField = 6,
    ToHand = 7,
    Discard = 8,
    ToDeck = 9,
    ToDeckBottom = 10,
    ToPrize = 11,
    NotMove = 12,
    DamageCounter = 13,
    AttachTo = 22,
    Look = 24,
    EffectTarget = 25,
    DiscardEnergyCard = 26,
    SwitchEnergyCard = 28,
    DiscardEnergy = 30,
    Attack = 35,
    DrawCount = 38,
    IsFirst = 41,
    Mulligan = 42,
    Activate = 43,
    CoinHead = 46,
    /// Extension: deck shuffle order (answer with `answer_shuffle`).
    Shuffle = 49,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OptionType {
    Number = 0,
    Yes = 1,
    No = 2,
    Card = 3,
    ToolCard = 4,
    EnergyCard = 5,
    Energy = 6,
    Play = 7,
    Attach = 8,
    Evolve = 9,
    Ability = 10,
    Discard = 11,
    Retreat = 12,
    Attack = 13,
    End = 14,
    Skill = 15,
    SpecialCondition = 16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AreaType {
    Deck = 1,
    Hand = 2,
    Discard = 3,
    Active = 4,
    Bench = 5,
    Prize = 6,
    Stadium = 7,
    Energy = 8,
    Tool = 9,
    PreEvolution = 10,
    Player = 11,
    Looking = 12,
}

/// One option (cabt `Option`). `card_id` is the card-database index,
/// `serial` the card instance.
#[derive(Clone, Debug, Default)]
pub struct Opt {
    pub kind: u8,
    pub number: Option<i32>,
    pub area: Option<u8>,
    pub index: Option<u8>,
    pub player_index: Option<u8>,
    pub in_play_area: Option<u8>,
    pub in_play_index: Option<u8>,
    pub attack_id: Option<u8>,
    pub card_id: Option<u16>,
    pub serial: Option<u8>,
}

#[derive(Clone, Debug)]
pub struct SelectData {
    /// Deciding player index.
    pub player: u8,
    pub select_type: SelectType,
    pub context: SelectContext,
    pub min_count: usize,
    pub max_count: usize,
    pub options: Vec<Opt>,
    /// Internal: how option indices map back to answers.
    source: Source,
}

impl SelectData {
    /// The oracle wire answer for option indices `chosen` (the `a` field of a
    /// trace step), for prompt selects.
    pub fn wire_answer(&self, chosen: &[usize]) -> Result<Value, GameError> {
        raw_answer(self, chosen)
    }

    /// Whether `wire_answer(chosen)` would succeed (every pick names an option; a single-valued prompt
    /// needs one pick), without building it.
    pub fn picks_ok(&self, chosen: &[usize]) -> bool {
        match &self.source {
            Source::Prompt(_, vals, AnswerShape::Single) => chosen.first().is_some_and(|k| *k < vals.len()),
            Source::Prompt(_, vals, _) => chosen.iter().all(|k| *k < vals.len()),
            _ => false,
        }
    }

    /// True when the same option may be picked more than once (each pick is
    /// one damage counter: Put / Move / Remove damage prompts).
    /// The prompt message this select answers (empty for turn / chance selects).
    pub fn prompt_message(&self, g: &Game) -> &'static str {
        match &self.source {
            Source::Prompt(k, _, _) => g.prompts.as_slice()[*k].message,
            _ => "",
        }
    }

    pub fn allows_repeats(&self) -> bool {
        self.context == SelectContext::DamageCounter
    }
}

#[derive(Clone, Debug)]
enum Source {
    Turn(Vec<TurnOption>),
    /// Chance prompt index (manual chance).
    Chance(usize),
    /// Prompt index plus the wire value per option.
    Prompt(usize, Vec<Wire>, AnswerShape),
}

/// The wire answer of one option. The common kinds are kept typed so a pick can be decoded without
/// building JSON; [`Wire::json`] makes the value the oracle's trace has.
#[derive(Clone, Debug)]
enum Wire {
    /// An index into the prompt's list.
    Idx(usize),
    Target(CardTarget),
    Bool(bool),
    Json(Value),
}

impl Wire {
    fn json(&self) -> Value {
        match self {
            Wire::Idx(i) => json!(i),
            Wire::Target(t) => target_json(*t),
            Wire::Bool(b) => json!(b),
            Wire::Json(v) => v.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AnswerShape {
    /// Answer is the JSON array of chosen option values.
    Array,
    /// Answer is the single chosen value.
    Single,
    /// PutDamage: picks may repeat; each pick places `multiple` damage on
    /// that option's target (values are target objects).
    Counters(i32),
}

fn context_for(message: &str) -> SelectContext {
    match message {
        "CHOOSE_STARTING_POKEMONS" => SelectContext::SetupActivePokemon,
        "CHOOSE_NEW_ACTIVE_POKEMON" | "CHOOSE_POKEMON_TO_SWITCH" => SelectContext::Switch,
        "CHOOSE_PRIZE_CARD" => SelectContext::ToHand,
        "CHOOSE_PRIZE_CARD_TO_DISCARD" => SelectContext::Discard,
        "CHOOSE_ENERGY_TO_PAY_RETREAT_COST" | "CHOOSE_ENERGIES_TO_DISCARD" => SelectContext::DiscardEnergy,
        "WANT_TO_DRAW_CARDS" => SelectContext::DrawCount,
        "GO_FIRST" => SelectContext::IsFirst,
        "WANT_TO_USE_ABILITY" => SelectContext::Activate,
        m if m.contains("DISCARD") => SelectContext::Discard,
        m if m.contains("HAND") => SelectContext::ToHand,
        m if m.contains("BENCH") => SelectContext::ToBench,
        m if m.contains("DECK") => SelectContext::ToDeck,
        _ => SelectContext::EffectTarget,
    }
}

fn area_of(r: ListRef) -> (u8, u8) {
    match r {
        ListRef::Deck(p) => (AreaType::Deck as u8, p),
        ListRef::Hand(p) => (AreaType::Hand as u8, p),
        ListRef::Discard(p) => (AreaType::Discard as u8, p),
        ListRef::Prize(p, _) => (AreaType::Prize as u8, p),
        ListRef::Stadium(p) => (AreaType::Stadium as u8, p),
        ListRef::Slot(p, _) | ListRef::SlotEnergies(p, _) => (AreaType::Bench as u8, p),
        _ => (AreaType::Looking as u8, 0),
    }
}

impl Game {
    /// The current decision as a cabt-style select, or `None` when the game
    /// is over (chance and info prompts are settled internally first).
    pub fn select(&mut self) -> Result<Option<SelectData>, GameError> {
        self.select_with(false)
    }

    /// Like [`select`], but with `manual_chance` coin flips and shuffles
    /// requested by prompts are returned to the caller instead of drawn from
    /// the internal RNG. (Coin flips resolved inline by effects still use the
    /// RNG; control them with [`Game::rng`].)
    pub fn select_with(&mut self, manual_chance: bool) -> Result<Option<SelectData>, GameError> {
        if manual_chance {
            loop {
                match self.pending() {
                    Pending::Info(i) => self.resolve(i, Res::True)?,
                    Pending::Chance(i) => return Ok(Some(self.chance_select(i))),
                    _ => break,
                }
            }
        } else {
            self.settle()?;
        }
        match self.pending() {
            Pending::Finished | Pending::Stuck => Ok(None),
            Pending::Turn(p) => Ok(Some(self.turn_select(p))),
            Pending::Decision(i) => Ok(Some(self.prompt_select(i))),
            Pending::Chance(_) | Pending::Info(_) => unreachable!("settled"),
        }
    }

    fn chance_select(&self, i: usize) -> SelectData {
        let pr = self.prompts.as_slice()[i];
        let player = self.st.player_index_by_id(pr.player_id) as u8;
        match pr.kind {
            PromptKind::CoinFlip => SelectData {
                player,
                select_type: SelectType::YesNo,
                context: SelectContext::CoinHead,
                min_count: 1,
                max_count: 1,
                options: vec![Opt { kind: OptionType::Yes as u8, ..Default::default() }, Opt { kind: OptionType::No as u8, ..Default::default() }],
                source: Source::Chance(i),
            },
            _ => SelectData {
                player,
                select_type: SelectType::Count,
                context: SelectContext::Shuffle,
                min_count: 0,
                max_count: 0,
                options: vec![],
                source: Source::Chance(i),
            },
        }
    }

    /// Answer a manual shuffle: `order` is a permutation of the deck indices.
    pub fn answer_shuffle(&mut self, sel: &SelectData, order: &[u8]) -> R {
        match sel.source {
            Source::Chance(i) => self.resolve(i, Res::Order(crate::list::List::from_slice(order))),
            _ => Err(GameError("NOT_A_SHUFFLE")),
        }
    }

    fn turn_select(&self, p: u8) -> SelectData {
        let opts = crate::options::legal_actions(self);
        let options = opts
            .iter()
            .map(|o| {
                let mut r = Opt::default();
                match o.action {
                    Action::PlayCard { hand_index, target } => {
                        let c = self.st.players[p as usize].hand.as_slice()[hand_index as usize];
                        let d = self.st.cdef(c);
                        r.kind = if d.is_energy() || (d.is_trainer() && d.trainer_type == TrainerType::Tool as u8) {
                            OptionType::Attach as u8
                        } else if d.is_pokemon() && target.slot != SlotType::Bench || (d.is_pokemon() && d.stage != Stage::Basic as u8) {
                            OptionType::Evolve as u8
                        } else {
                            OptionType::Play as u8
                        };
                        r.area = Some(AreaType::Hand as u8);
                        r.index = Some(hand_index);
                        r.serial = Some(c);
                        r.card_id = Some(self.st.cards[c as usize].def);
                        if target.slot == SlotType::Active || target.slot == SlotType::Bench {
                            r.in_play_area = Some(if target.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 });
                            r.in_play_index = Some(target.index);
                        }
                    }
                    Action::Attack { name, from } => {
                        r.kind = OptionType::Attack as u8;
                        // An attack copied from a Benched Pokemon (Memory Helix): the source card and its attack.
                        let source = from.and_then(|f| {
                            let pl = &self.st.players[p as usize];
                            pl.bench.iter().filter_map(|b| self.st.slot_pokemon(p as usize, *b)).find(|c| self.st.cdef(*c).full_name == f)
                        });
                        if let Some(c) = source {
                            r.attack_id = self.st.cdef(c).attacks.iter().position(|a| a.name == name).map(|i| i as u8);
                            r.card_id = Some(self.st.cards[c as usize].def);
                        } else if let Some(c) = self.st.active_pokemon(p as usize) {
                            r.attack_id = self.st.cdef(c).attacks.iter().position(|a| a.name == name).map(|i| i as u8);
                            r.card_id = Some(self.st.cards[c as usize].def);
                        }
                    }
                    Action::UseAbility { target, .. } => {
                        r.kind = OptionType::Ability as u8;
                        r.area = Some(if target.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 });
                        r.index = Some(target.index);
                    }
                    Action::Retreat { bench_index } => {
                        r.kind = OptionType::Retreat as u8;
                        r.area = Some(AreaType::Bench as u8);
                        r.index = Some(bench_index);
                    }
                    Action::UseStadium => {
                        r.kind = OptionType::Skill as u8;
                        r.area = Some(AreaType::Stadium as u8);
                    }
                    Action::UseTrainerAbility { .. } => r.kind = OptionType::Skill as u8,
                    Action::Pass => r.kind = OptionType::End as u8,
                }
                r.player_index = Some(p);
                r
            })
            .collect();
        SelectData {
            player: p,
            select_type: SelectType::Main,
            context: SelectContext::Main,
            min_count: 1,
            max_count: 1,
            options,
            source: Source::Turn(opts),
        }
    }

    fn prompt_select(&self, i: usize) -> SelectData {
        let pr = self.prompts.as_slice()[i];
        let player = self.st.player_index_by_id(pr.player_id) as u8;
        let persp = self.st.player_index_by_id(pr.perspective_id());
        let ctx = context_for(pr.message);
        let mk = |t: SelectType, c: SelectContext, min: usize, max: usize, options: Vec<Opt>, vals: Vec<Wire>, sh: AnswerShape| SelectData {
            player,
            select_type: t,
            context: c,
            min_count: min,
            max_count: max,
            options,
            source: Source::Prompt(i, vals, sh),
        };
        match pr.kind {
            PromptKind::Confirm => {
                let o = |k: OptionType| Opt { kind: k as u8, ..Default::default() };
                mk(SelectType::YesNo, ctx, 1, 1, vec![o(OptionType::Yes), o(OptionType::No)], vec![Wire::Bool(true), Wire::Bool(false)], AnswerShape::Single)
            }
            PromptKind::Select { values, .. } => {
                let n = values.len();
                let options = (0..n)
                    .map(|k| Opt {
                        kind: OptionType::Number as u8,
                        number: Some(match values {
                            SelectValues::DrawCards(m) => m as i32 - k as i32,
                            _ => k as i32,
                        }),
                        ..Default::default()
                    })
                    .collect();
                mk(SelectType::Count, ctx, 1, 1, options, (0..n).map(Wire::Idx).collect(), AnswerShape::Single)
            }
            PromptKind::ChooseCards { cards, filter, opts } => {
                let list = self.prompt_list(cards);
                let sel = self.choose_cards_selectable(cards, &filter, &opts);
                let (area, owner) = area_of(cards);
                let mut options = Vec::new();
                let mut vals = Vec::new();
                for (k, &c) in list.iter().enumerate() {
                    if sel[k] {
                        options.push(Opt {
                            kind: OptionType::Card as u8,
                            area: Some(area),
                            index: Some(k as u8),
                            player_index: Some(owner),
                            card_id: Some(self.st.cards[c as usize].def),
                            serial: Some(c),
                            ..Default::default()
                        });
                        vals.push(Wire::Idx(k));
                    }
                }
                let min = opts.min as usize;
                let max = (opts.max as usize).min(options.len());
                mk(SelectType::Card, ctx, if opts.allow_cancel { 0 } else { min }, max, options, vals, AnswerShape::Array)
            }
            PromptKind::ChoosePokemon { min, max, .. } => {
                let cands = self.choose_pokemon_candidates(&pr);
                let options = cands
                    .iter()
                    .map(|t| {
                        let owner = if t.player == PlayerType::BottomPlayer { persp } else { 1 - persp };
                        let s = get_target(&self.st, persp, *t).unwrap();
                        let c = self.st.slot_pokemon(s.p as usize, s.s);
                        Opt {
                            kind: OptionType::Card as u8,
                            area: Some(if t.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                            index: Some(t.index),
                            player_index: Some(owner as u8),
                            card_id: c.map(|c| self.st.cards[c as usize].def),
                            serial: c,
                            ..Default::default()
                        }
                    })
                    .collect::<Vec<_>>();
                let vals = cands.iter().map(|t| Wire::Target(*t)).collect();
                let n = options.len();
                mk(SelectType::Card, ctx, (min as usize).min(n), (max as usize).min(n), options, vals, AnswerShape::Array)
            }
            PromptKind::ChoosePrize { count, use_opponent_prizes, .. } => {
                let q = if use_opponent_prizes { 1 - persp } else { persp };
                let left = self.st.players[q].prizes.iter().filter(|l| !l.is_empty()).count();
                let options = (0..left)
                    .map(|k| Opt { kind: OptionType::Card as u8, area: Some(AreaType::Prize as u8), index: Some(k as u8), player_index: Some(q as u8), ..Default::default() })
                    .collect();
                let n = (count as usize).min(left);
                mk(SelectType::Card, ctx, n, n, options, (0..left).map(Wire::Idx).collect(), AnswerShape::Array)
            }
            PromptKind::ChooseEnergy { energy, cost, .. } => {
                let options = energy
                    .iter()
                    .enumerate()
                    .map(|(k, e)| Opt {
                        kind: OptionType::Energy as u8,
                        area: Some(AreaType::Energy as u8),
                        index: Some(k as u8),
                        card_id: Some(self.st.cards[e.card as usize].def),
                        serial: Some(e.card),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>();
                let n = options.len();
                // A payment never has more cards than the cost (ruling 1652); a larger bound made the
                // pick-mask search run out of budget and offer cards that lead to a dead end.
                let max = n.min(cost.len().max(1));
                mk(SelectType::Energy, ctx, 0, max, options, (0..n).map(Wire::Idx).collect(), AnswerShape::Array)
            }
            PromptKind::AttachEnergy { cards, player_type, slots, filter, o } => {
                // One option per (energy card, target) pair; pick each energy at most once.
                let list = self.prompt_list(cards);
                let targets: Vec<CardTarget> = slot_targets(&self.st, persp, player_type, slots.as_slice())
                    .into_iter()
                    .filter(|t| !o.blocked_to.iter().any(|b| b.player == t.player && b.slot == t.slot && b.index == t.index))
                    .collect();
                let (mut options, mut vals) = (Vec::new(), Vec::new());
                for (k, &c) in list.iter().enumerate() {
                    let d = self.st.cdef(c);
                    if !(d.is_energy() && !o.blocked.contains(&(k as u8)) && filter.matches(d)) {
                        continue;
                    }
                    for t in &targets {
                        options.push(Opt {
                            kind: OptionType::EnergyCard as u8,
                            area: Some(area_of(cards).0),
                            index: Some(k as u8),
                            in_play_area: Some(if t.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                            in_play_index: Some(t.index),
                            player_index: Some(if t.player == PlayerType::BottomPlayer { persp as u8 } else { 1 - persp as u8 }),
                            card_id: Some(self.st.cards[c as usize].def),
                            serial: Some(c),
                            ..Default::default()
                        });
                        vals.push(Wire::Json(json!({ "to": target_json(*t), "index": k })));
                    }
                }
                mk(SelectType::AttachedCard, SelectContext::AttachTo, o.min as usize, o.max as usize, options, vals, AnswerShape::Array)
            }
            PromptKind::DiscardEnergy { player_type, slots, filter, o } | PromptKind::MoveEnergy { player_type, slots, filter, o } => {
                let is_move = matches!(pr.kind, PromptKind::MoveEnergy { .. });
                let all = slot_targets(&self.st, persp, player_type, slots.as_slice());
                let (mut options, mut vals) = (Vec::new(), Vec::new());
                for (from, idx) in self.energy_sources(persp, player_type, slots.as_slice(), &filter, &o) {
                    let s = get_target(&self.st, persp, from).unwrap();
                    for i in idx {
                        let slot = self.st.slot(s.p as usize, s.s);
                        let c = if crate::prompts::is_tool_filter(&filter) { slot.tools.as_slice()[i as usize] } else { slot.cards.as_slice()[i as usize] };
                        let mut base = Opt {
                            kind: OptionType::EnergyCard as u8,
                            area: Some(if from.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                            index: Some(from.index),
                            player_index: Some(s.p),
                            card_id: Some(self.st.cards[c as usize].def),
                            serial: Some(c),
                            ..Default::default()
                        };
                        if is_move {
                            for to in all.iter().filter(|t| !(t.player == from.player && t.slot == from.slot && t.index == from.index)) {
                                base.in_play_area = Some(if to.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 });
                                base.in_play_index = Some(to.index);
                                options.push(base.clone());
                                vals.push(Wire::Json(json!({ "from": target_json(from), "to": target_json(*to), "index": i })));
                            }
                        } else {
                            options.push(base);
                            vals.push(Wire::Json(json!({ "from": target_json(from), "index": i })));
                        }
                    }
                }
                let max = o.max.map(|m| m as usize).unwrap_or(options.len());
                let ctx = if is_move { SelectContext::SwitchEnergyCard } else { SelectContext::DiscardEnergyCard };
                mk(SelectType::AttachedCard, ctx, o.min as usize, max, options, vals, AnswerShape::Array)
            }
            PromptKind::PutDamage { player_type, slots, damage, blocked, allow_partial, damage_multiple, .. } => {
                let bl: Vec<CardTarget> = blocked.iter().copied().collect();
                let targets: Vec<CardTarget> = slot_targets(&self.st, persp, player_type, slots.as_slice())
                    .into_iter()
                    .filter(|t| !bl.iter().any(|b| get_target(&self.st, persp, *b).ok() == get_target(&self.st, persp, *t).ok()))
                    .collect();
                let options = targets
                    .iter()
                    .map(|t| Opt {
                        kind: OptionType::Card as u8,
                        area: Some(if t.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                        index: Some(t.index),
                        player_index: Some(if t.player == PlayerType::BottomPlayer { persp as u8 } else { 1 - persp as u8 }),
                        ..Default::default()
                    })
                    .collect();
                let n = (damage / damage_multiple.max(1)) as usize;
                let vals = targets.iter().map(|t| Wire::Target(*t)).collect();
                mk(SelectType::Card, SelectContext::DamageCounter, if allow_partial { 0 } else { n }, n, options, vals, AnswerShape::Counters(damage_multiple))
            }
            PromptKind::MoveDamage { player_type, slots, o, .. } | PromptKind::RemoveDamage { player_type, slots, o, .. } => {
                let all = slot_targets(&self.st, persp, player_type, slots.as_slice());
                let (mut options, mut vals) = (Vec::new(), Vec::new());
                for f in &all {
                    for t in all.iter().filter(|t| !(t.player == f.player && t.slot == f.slot && t.index == f.index)) {
                        options.push(Opt {
                            kind: OptionType::Card as u8,
                            area: Some(if f.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                            index: Some(f.index),
                            in_play_area: Some(if t.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                            in_play_index: Some(t.index),
                            ..Default::default()
                        });
                        vals.push(Wire::Json(json!({ "from": target_json(*f), "to": target_json(*t) })));
                    }
                }
                let max = o.max.map(|m| m as usize).unwrap_or(12);
                mk(SelectType::Card, SelectContext::DamageCounter, o.min as usize, max, options, vals, AnswerShape::Array)
            }
            PromptKind::OrderCards { cards, .. } => {
                let list = self.prompt_list(cards);
                let options = list
                    .iter()
                    .enumerate()
                    .map(|(k, c)| Opt { kind: OptionType::Card as u8, index: Some(k as u8), card_id: Some(self.st.cards[*c as usize].def), serial: Some(*c), ..Default::default() })
                    .collect::<Vec<_>>();
                let n = options.len();
                mk(SelectType::Card, SelectContext::ToDeck, n, n, options, (0..n).map(Wire::Idx).collect(), AnswerShape::Array)
            }
            PromptKind::SelectOption { values, disabled, .. } => {
                let (mut options, mut vals) = (Vec::new(), Vec::new());
                for k in 0..values.len() {
                    if disabled.map(|d| d & (1 << k) != 0).unwrap_or(false) {
                        continue;
                    }
                    options.push(Opt { kind: OptionType::Number as u8, number: Some(k as i32), ..Default::default() });
                    vals.push(Wire::Idx(k));
                }
                mk(SelectType::Count, ctx, 1, 1, options, vals, AnswerShape::Single)
            }
            PromptKind::ChooseAttack { cards, .. } => {
                let (mut options, mut vals) = (Vec::new(), Vec::new());
                for (i, c) in cards.iter().enumerate() {
                    for (a, at) in self.st.cdef(*c).attacks.iter().enumerate() {
                        options.push(Opt {
                            kind: OptionType::Attack as u8,
                            attack_id: Some(a as u8),
                            card_id: Some(self.st.cards[*c as usize].def),
                            serial: Some(*c),
                            ..Default::default()
                        });
                        vals.push(Wire::Json(json!({ "index": i, "attack": at.name })));
                    }
                }
                mk(SelectType::Attack, SelectContext::Attack, 1, 1, options, vals, AnswerShape::Single)
            }
            _ => mk(SelectType::YesNo, SelectContext::Look, 0, 0, vec![], vec![], AnswerShape::Single),
        }
    }

    /// Answer a select with option indices. Invalid combinations (e.g. an
    /// energy set that does not pay the cost exactly) return an error and
    /// leave the game unchanged.
    pub fn answer(&mut self, sel: &SelectData, chosen: &[usize]) -> R {
        self.answer_with(sel, chosen, true)
    }

    /// [`Game::answer`]; `rollback` false skips the backup copy that restores the game when the answer
    /// is rejected (a caller working on a copy it throws away).
    fn answer_with(&mut self, sel: &SelectData, chosen: &[usize], rollback: bool) -> R {
        match &sel.source {
            Source::Chance(i) => {
                let k = *chosen.first().ok_or(GameError("EMPTY_ANSWER"))?;
                match self.prompts.as_slice()[*i].kind {
                    PromptKind::CoinFlip => self.resolve(*i, Res::Bool(k == 0)),
                    _ => {
                        let r = self.draw_chance(*i);
                        self.resolve(*i, r)
                    }
                }
            }
            Source::Turn(opts) => {
                let k = *chosen.first().ok_or(GameError("EMPTY_ANSWER"))?;
                let a = opts.get(k).ok_or(GameError("BAD_OPTION"))?.action;
                self.act(a)
            }
            Source::Prompt(i, _, _) => {
                let pr = self.prompts.as_slice()[*i];
                let res = self.decode_chosen(&pr, sel, chosen)?;
                if !rollback {
                    return self.resolve(*i, res);
                }
                let backup = self.fork();
                let r = self.resolve(*i, res);
                if r.is_err() {
                    self.copy_from(&backup);
                }
                r
            }
        }
    }
}

/// Twinleaf wire answer for option indices `chosen` of a prompt select.
fn raw_answer(sel: &SelectData, chosen: &[usize]) -> Result<Value, GameError> {
    let (vals, shape) = match &sel.source {
        Source::Prompt(_, vals, shape) => (vals, shape),
        _ => return Err(GameError("NOT_A_PROMPT")),
    };
    Ok(match shape {
                    AnswerShape::Single => vals.get(*chosen.first().ok_or(GameError("EMPTY_ANSWER"))?).map(Wire::json).ok_or(GameError("BAD_OPTION"))?,
                    AnswerShape::Array => {
                        let mut v = Vec::new();
                        for &k in chosen {
                            v.push(vals.get(k).map(Wire::json).ok_or(GameError("BAD_OPTION"))?);
                        }
                        Value::Array(v)
                    }
                    AnswerShape::Counters(mult) => {
                        let mut agg: Vec<(usize, i32)> = Vec::new();
                        for &k in chosen {
                            if k >= vals.len() {
                                return Err(GameError("BAD_OPTION"));
                            }
                            match agg.iter_mut().find(|x| x.0 == k) {
                                Some(x) => x.1 += *mult,
                                None => agg.push((k, *mult)),
                            }
                        }
                        Value::Array(agg.into_iter().map(|(k, d)| json!({ "target": vals[k].json(), "damage": d })).collect())
                    }
                })
}

impl Game {
    /// Whether `chosen` would be accepted by the prompt's decode + validate
    /// (pure: the game is not modified).
    pub fn is_valid_answer(&self, sel: &SelectData, chosen: &[usize]) -> bool {
        match &sel.source {
            Source::Prompt(i, _, _) => {
                let pr = self.prompts.as_slice()[*i];
                self.decode_chosen(&pr, sel, chosen).is_ok()
            }
            Source::Turn(opts) => chosen.len() == 1 && chosen[0] < opts.len(),
            Source::Chance(_) => chosen.len() == 1,
        }
    }

    /// Whether some superset of `chosen` could still pass decode + validate.
    /// For the prompt kinds whose checks (other than the minimum count) only
    /// get stricter as picks are added, this is validity with the minimum
    /// dropped; other kinds are assumed extendable.
    fn extendable(&self, sel: &SelectData, chosen: &[usize]) -> bool {
        let i = match &sel.source {
            Source::Prompt(i, _, _) => *i,
            _ => return true,
        };
        let mut pr = self.prompts.as_slice()[i];
        match &mut pr.kind {
            PromptKind::ChooseCards { opts, .. } => opts.min = 0,
            PromptKind::ChoosePokemon { min, .. } => *min = 0,
            PromptKind::AttachEnergy { o, .. } => o.min = 0,
            PromptKind::DiscardEnergy { o, .. } | PromptKind::MoveEnergy { o, .. } => o.min = 0,
            PromptKind::MoveDamage { o, .. } | PromptKind::RemoveDamage { o, .. } => o.min = 0,
            PromptKind::PutDamage { allow_partial, .. } => *allow_partial = true,
            PromptKind::ChooseEnergy { cost, .. } => {
                // Payments never have more cards than the cost (ruling 1652): longer picks cannot be completed.
                return sel.picks_ok(chosen) && chosen.len() <= cost.len();
            }
            _ => return true,
        }
        self.decode_chosen(&pr, sel, chosen).is_ok()
    }

    /// `is_valid_answer` plus resolving it on a copy: card callbacks can reject
    /// answers the prompt's own validate accepts.
    pub fn is_accepted_answer(&self, sel: &SelectData, chosen: &[usize]) -> bool {
        if !self.is_valid_answer(sel, chosen) {
            return false;
        }
        let mut trial = self.fork();
        trial.answer_with(sel, chosen, false).is_ok()
    }

    /// The decoded and validated answer for option indices `chosen` of a prompt select (what
    /// [`Game::answer`] resolves), without resolving it.
    pub fn decode_picks(&self, sel: &SelectData, chosen: &[usize]) -> Result<Res, GameError> {
        let Source::Prompt(i, _, _) = &sel.source else { return Err(GameError("NOT_A_PROMPT")) };
        let pr = self.prompts.as_slice()[*i];
        self.decode_chosen(&pr, sel, chosen)
    }

    /// `raw_answer` then `decode_answer` for option indices `chosen` (same results and errors), without
    /// building the JSON answer for the common prompt kinds.
    fn decode_chosen(&self, pr: &PromptRec, sel: &SelectData, chosen: &[usize]) -> Result<Res, GameError> {
        let r = self.decode_chosen_typed(pr, sel, chosen);
        if crate::game::verify_cache() {
            let json = raw_answer(sel, chosen).and_then(|raw| self.decode_answer(pr, &raw));
            assert_eq!(format!("{:?}", r), format!("{:?}", json), "typed decode differs from the JSON decode of {:?}", chosen);
        }
        r
    }

    fn decode_chosen_typed(&self, pr: &PromptRec, sel: &SelectData, chosen: &[usize]) -> Result<Res, GameError> {
        let Source::Prompt(_, vals, shape) = &sel.source else { return Err(GameError("NOT_A_PROMPT")) };
        // The errors `raw_answer` gives.
        match shape {
            AnswerShape::Single => {
                let k = *chosen.first().ok_or(GameError("EMPTY_ANSWER"))?;
                if k >= vals.len() {
                    return Err(GameError("BAD_OPTION"));
                }
            }
            _ => {
                if chosen.iter().any(|&k| k >= vals.len()) {
                    return Err(GameError("BAD_OPTION"));
                }
            }
        }
        let idx = |k: &usize| match &vals[*k] {
            Wire::Idx(i) => Some(*i),
            _ => None,
        };
        let target = |k: &usize| match &vals[*k] {
            Wire::Target(t) => Some(*t),
            _ => None,
        };
        match (&pr.kind, shape) {
            (PromptKind::ChooseCards { cards, filter, opts }, AnswerShape::Array) => self.decode_choose_cards(*cards, filter, opts, chosen.iter().map(idx)),
            (PromptKind::ChoosePokemon { .. }, AnswerShape::Array) => self.decode_choose_pokemon(pr, chosen.iter().map(target)),
            (PromptKind::ChoosePrize { .. }, AnswerShape::Array) => self.decode_choose_prize(pr, chosen.iter().map(idx)),
            (PromptKind::ChooseEnergy { energy, cost, .. }, AnswerShape::Array) => self.decode_choose_energy(energy, cost, chosen.iter().map(idx)),
            (PromptKind::Confirm, AnswerShape::Single) => match &vals[chosen[0]] {
                Wire::Bool(b) => Ok(Res::Bool(*b)),
                _ => Err(GameError("INVALID_PROMPT_RESULT")),
            },
            (PromptKind::Select { .. } | PromptKind::SelectOption { .. }, AnswerShape::Single) => match &vals[chosen[0]] {
                Wire::Idx(i) => Ok(Res::Int(*i as i32)),
                _ => Err(GameError("INVALID_PROMPT_RESULT")),
            },
            _ => {
                let raw = raw_answer(sel, chosen)?;
                self.decode_answer(pr, &raw)
            }
        }
    }

    /// For a pick-by-pick answer: which options can be picked next so that a
    /// valid answer is still reachable, and whether `picks` can be submitted
    /// as is (STOP). Bounded search; if the bound is hit an option is assumed
    /// reachable.
    pub fn pick_mask(&self, sel: &SelectData, picks: &[usize]) -> (Vec<bool>, bool) {
        let n = sel.options.len();
        if !matches!(sel.source, Source::Prompt(..)) {
            return (vec![true; n], false);
        }
        if sel.max_count <= 1 {
            // Single pick: offer options the prompt and the card's callback accept.
            let mask: Vec<bool> = (0..n).map(|j| self.is_accepted_answer(sel, &[j])).collect();
            let stop = sel.min_count == 0 && self.is_accepted_answer(sel, &[]);
            return (mask, stop);
        }
        let repeats = sel.allows_repeats();
        let exact = match &sel.source {
            Source::Prompt(i, _, _) => matches!(self.prompts.as_slice()[*i].kind, PromptKind::ChooseEnergy { .. }),
            _ => false,
        };
        let stop = picks.len() >= sel.min_count && self.is_accepted_answer(sel, picks);
        let mut mask = vec![false; n];
        if picks.len() >= sel.max_count {
            return (mask, stop);
        }
        let mut buf: Vec<usize> = picks.to_vec();
        for j in 0..n {
            if !repeats && picks.contains(&j) {
                continue;
            }
            buf.push(j);
            // Each option gets its own small search budget, so a large option
            // list (e.g. every from/to pair of a move-damage prompt) cannot
            // exhaust it for the options that come last. Energy payments search
            // exactly: their picks are few (at most the cost) and an exhausted
            // budget would offer an Energy that can't complete the payment.
            let mut budget = if exact { usize::MAX } else { 64usize };
            mask[j] = self.completable(sel, &mut buf, 0, repeats, &mut budget);
            buf.pop();
        }
        (mask, stop)
    }

    /// Depth-first search for a valid superset of `buf`; the added options are
    /// enumerated in index order from `start` (combinations, not permutations).
    fn completable(&self, sel: &SelectData, buf: &mut Vec<usize>, start: usize, repeats: bool, budget: &mut usize) -> bool {
        if *budget == 0 {
            return true;
        }
        *budget -= 1;
        if !self.extendable(sel, buf) {
            return false;
        }
        if buf.len() >= sel.min_count && self.is_valid_answer(sel, buf) {
            let mut trial = self.fork();
            if trial.answer_with(sel, buf, false).is_ok() {
                return true;
            }
        }
        if buf.len() >= sel.max_count {
            return false;
        }
        for k in start..sel.options.len() {
            if !repeats && buf.contains(&k) {
                continue;
            }
            buf.push(k);
            let ok = self.completable(sel, buf, if repeats { k } else { k + 1 }, repeats, budget);
            buf.pop();
            if ok {
                return true;
            }
            if *budget == 0 {
                return true;
            }
        }
        false
    }
}
