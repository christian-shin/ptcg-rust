//! Engine-driven selection interface (the selection interface a host plays through):
//! every decision is a [`SelectData`] with `type`, `context`, `min_count`,
//! `max_count` and an explicit `option` list; the agent answers with option
//! indices.
//!
//! Answers are translated to the wire answers of the recorded traces and go through the same decode + validate path,
//! so the interface cannot accept anything the rules engine would reject.

use crate::game::{Action, Game, GameError, Pending, R};
use crate::prompts::*;
use crate::state::ListRef;
use crate::types::*;
use crate::list::{CardId, CardList, SVec};
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
    Damage = 15,
    Heal = 17,
    EvolvesFrom = 18,
    EvolvesTo = 19,
    Devolve = 20,
    AttachFrom = 21,
    AttachTo = 22,
    Look = 24,
    EffectTarget = 25,
    DiscardEnergyCard = 26,
    SwitchEnergyCard = 28,
    DiscardEnergy = 30,
    ToHandEnergy = 31,
    SkillOrder = 34,
    Attack = 35,
    DisableAttack = 36,
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

/// One option of a select. `card_id` is the card-database index,
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
    /// The card instance whose effect the option is, for the options of an order-of-effects question (not in the
    /// observation features).
    pub source: Option<u8>,
    /// The player at the two ends of a damage counter move or remove option (absolute seats): the Pokémon the counter
    /// leaves (`area` / `index`) and the one it goes to (`in_play_area` / `in_play_index`). Not in the observation
    /// features (`player_index` stays unset for these options).
    pub from_player: Option<u8>,
    pub to_player: Option<u8>,
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
    /// The card whose effect asks and the card the question is about (`Provenance::NONE` for a turn decision, a chance
    /// or a rules question).
    pub provenance: Provenance,
    /// Internal: how option indices map back to answers.
    map: Source,
    /// Per option of a prompt select, the answer it stands for.
    vals: Vec<OptionValue>,
    /// Per option of a turn select, its action.
    actions: Vec<Action>,
}

impl Default for SelectData {
    /// A select with no option (answering it is an error): a buffer for [`Game::select_into`].
    fn default() -> Self {
        SelectData {
            player: 0,
            select_type: SelectType::Main,
            context: SelectContext::Main,
            min_count: 0,
            max_count: 0,
            options: Vec::new(),
            provenance: Provenance::NONE,
            map: Source::Turn,
            vals: Vec::new(),
            actions: Vec::new(),
        }
    }
}

impl SelectData {
    /// The wire answer for option indices `chosen` (the `a` field of a
    /// trace step), for prompt selects.
    pub fn wire_answer(&self, chosen: &[usize]) -> Result<Value, GameError> {
        raw_answer(self, chosen)
    }

    /// Whether `wire_answer(chosen)` would succeed (every pick names an option; a single-valued prompt
    /// needs one pick), without building it.
    pub fn picks_ok(&self, chosen: &[usize]) -> bool {
        match &self.map {
            Source::Prompt(_, AnswerShape::Single) => chosen.first().is_some_and(|k| *k < self.vals.len()),
            Source::Prompt(..) => chosen.iter().all(|k| *k < self.vals.len()),
            _ => false,
        }
    }

    /// What option `j` of a prompt select answers, in the engine's own types (`None` for a turn or chance select, or no
    /// such option): the typed form of the option's part of [`SelectData::wire_answer`].
    pub fn option_value(&self, j: usize) -> Option<OptionValue> {
        match &self.map {
            Source::Prompt(..) => self.vals.get(j).copied(),
            _ => None,
        }
    }

    /// The prompt message this select answers (empty for turn / chance selects).
    pub fn prompt_message(&self, g: &Game) -> &'static str {
        match &self.map {
            Source::Prompt(k, _) => g.prompts.as_slice()[*k].purpose.message(),
            _ => "",
        }
    }

    /// True when the same option may be picked more than once (each pick is
    /// one damage counter: Put / Move / Remove damage prompts).
    pub fn allows_repeats(&self) -> bool {
        self.context == SelectContext::DamageCounter
    }
}

#[derive(Clone, Copy, Debug)]
enum Source {
    /// A turn decision (the options' actions: `SelectData::actions`).
    Turn,
    /// Chance prompt index (manual chance).
    Chance(usize),
    /// Prompt index and the answer's shape (the options' values: `SelectData::vals`).
    Prompt(usize, AnswerShape),
}

/// What one option of a prompt answers, in the engine's own types: a pick is decoded from it without building JSON
/// ([`Game::answer`]); `json` makes the value a recorded trace holds (the wire form, [`SelectData::wire_answer`]).
/// Targets are relative to the deciding player (`BottomPlayer` is that player).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionValue {
    /// An index into the prompt's list (a card of the offered list, a Prize card, an Energy of the payment, a position, a
    /// number or a label of the choice).
    Index(usize),
    /// A Pokémon in play.
    Target(CardTarget),
    /// Yes / no.
    Bool(bool),
    /// An Energy card (index) attached to a target.
    AttachTo(CardTarget, usize),
    /// A card (index) of a slot.
    From(CardTarget, usize),
    /// A card (index) of a slot, moved to a target.
    FromTo(CardTarget, CardTarget, usize),
    /// A damage counter moved from one target to another.
    Pair(CardTarget, CardTarget),
    /// An attack: of the card at `index` of the prompt's list, its attack `attack` (the position in the card's attacks),
    /// named `name` (the wire form names it).
    Attack { index: usize, attack: u8, name: &'static str },
}

impl OptionValue {
    fn json(&self) -> Value {
        match self {
            OptionValue::Index(i) => json!(i),
            OptionValue::Target(t) => target_json(*t),
            OptionValue::Bool(b) => json!(b),
            OptionValue::AttachTo(t, i) => json!({ "to": target_json(*t), "index": i }),
            OptionValue::From(f, i) => json!({ "from": target_json(*f), "index": i }),
            OptionValue::FromTo(f, t, i) => json!({ "from": target_json(*f), "to": target_json(*t), "index": i }),
            OptionValue::Pair(f, t) => json!({ "from": target_json(*f), "to": target_json(*t) }),
            OptionValue::Attack { index, name, .. } => json!({ "index": index, "attack": name }),
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

/// The select context of a prompt purpose (the outcome the prompt asks for).
pub(crate) fn context_for(purpose: PromptPurpose) -> SelectContext {
    use PromptPurpose as P;
    match purpose {
        P::SetupActive => SelectContext::SetupActivePokemon,
        P::NewActive => SelectContext::ToActive,
        P::SwitchPokemon => SelectContext::Switch,
        P::TakePrize | P::PutIntoHand => SelectContext::ToHand,
        P::DiscardCard => SelectContext::Discard,
        P::DiscardEnergy => SelectContext::DiscardEnergy,
        P::DrawCount => SelectContext::DrawCount,
        P::ChooseFirstPlayer => SelectContext::IsFirst,
        P::Activate => SelectContext::Activate,
        P::PutOntoBench => SelectContext::ToBench,
        P::PutIntoDeck | P::OrderCards => SelectContext::ToDeck,
        P::PutOnDeckBottom => SelectContext::ToDeckBottom,
        P::FirstPlayerFlip | P::ConfusionFlip | P::BurnFlip | P::SleepFlip => SelectContext::CoinHead,
        P::Shuffle => SelectContext::Shuffle,
        P::Look | P::TurnPrizeFaceUp | P::LookAtPrize => SelectContext::Look,
        P::RevealCard => SelectContext::NotMove,
        P::MulliganYours | P::MulliganOpponents => SelectContext::Mulligan,
        P::EvolutionCard => SelectContext::EvolvesTo,
        P::EvolvePokemon => SelectContext::EvolvesFrom,
        P::Devolve => SelectContext::Devolve,
        P::AttachCard => SelectContext::AttachTo,
        P::AttachTo => SelectContext::AttachFrom,
        P::MoveEnergy => SelectContext::SwitchEnergyCard,
        P::MoveDamage | P::PutDamageCounters => SelectContext::DamageCounter,
        P::DamageTarget => SelectContext::Damage,
        P::HealTarget => SelectContext::Heal,
        P::OrderEffects => SelectContext::SkillOrder,
        P::CopyAttack => SelectContext::Attack,
        P::DisableAttack => SelectContext::DisableAttack,
        // Not decisions of a select, or a choice among the labels of one effect.
        P::Wait | P::ChooseOption | P::CopyEffect | P::EffectTarget | P::SwapPokemon => SelectContext::EffectTarget,
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
    /// The current decision as a select, or `None` when the game
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

    /// [`Game::select`] into a select the caller keeps: the same value, its lists reused (no allocation per decision
    /// once they have the room). `false` when the game is over (or stuck), `out` then unchanged.
    pub fn select_into(&mut self, out: &mut SelectData) -> Result<bool, GameError> {
        self.settle()?;
        match self.pending() {
            Pending::Finished | Pending::Stuck => Ok(false),
            Pending::Turn(p) => {
                self.turn_select_into(p, out);
                Ok(true)
            }
            Pending::Decision(i) => {
                self.prompt_select_into(i, out);
                Ok(true)
            }
            Pending::Chance(_) | Pending::Info(_) => unreachable!("settled"),
        }
    }

    fn chance_select(&self, i: usize) -> SelectData {
        let pr = self.prompts.as_slice()[i];
        let player = pr.player;
        match pr.kind {
            PromptKind::CoinFlip => SelectData {
                player,
                select_type: SelectType::YesNo,
                context: SelectContext::CoinHead,
                min_count: 1,
                max_count: 1,
                options: vec![Opt { kind: OptionType::Yes as u8, ..Default::default() }, Opt { kind: OptionType::No as u8, ..Default::default() }],
                provenance: Provenance::NONE,
                map: Source::Chance(i),
                vals: Vec::new(),
                actions: Vec::new(),
            },
            _ => SelectData {
                player,
                select_type: SelectType::Count,
                context: SelectContext::Shuffle,
                min_count: 0,
                max_count: 0,
                options: vec![],
                provenance: Provenance::NONE,
                map: Source::Chance(i),
                vals: Vec::new(),
                actions: Vec::new(),
            },
        }
    }

    /// Answer a manual shuffle: `order` is a permutation of the deck indices.
    pub fn answer_shuffle(&mut self, sel: &SelectData, order: &[u8]) -> R {
        match sel.map {
            Source::Chance(i) => self.resolve(i, Res::Order(crate::list::List::from_slice(order))),
            _ => Err(GameError("NOT_A_SHUFFLE")),
        }
    }

    fn turn_select(&mut self, p: u8) -> SelectData {
        let mut s = SelectData::default();
        self.turn_select_into(p, &mut s);
        s
    }

    /// The turn decision of `p` as a select, into `out` (its lists reused).
    fn turn_select_into(&mut self, p: u8, out: &mut SelectData) {
        out.vals.clear();
        crate::options::legal_actions_into(self, &mut out.actions);
        let SelectData { options, actions, .. } = &mut *out;
        options.clear();
        options.extend(actions.iter().map(|a| {
            let mut r = Opt::default();
            match *a {
                Action::PlayCard { hand_index, target } => {
                    let c = self.st.players[p as usize].hand().as_slice()[hand_index as usize];
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
                Action::Attack { attack, .. } => {
                    r.kind = OptionType::Attack as u8;
                    // The attack and the card that prints it (a Benched Pokemon's, when copied by Memory Helix).
                    r.attack_id = Some(attack.idx() as u8);
                    r.card_id = Some(self.st.cards[attack.card as usize].def);
                }
                Action::UseAbility { target, .. } => {
                    r.kind = OptionType::Ability as u8;
                    r.area = Some(if target.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 });
                    r.index = Some(target.index);
                }
                // The Benched Pokémon is chosen after the cost is paid (a SwitchPokemon prompt, APR A-03).
                Action::Retreat => r.kind = OptionType::Retreat as u8,
                Action::UseStadium => {
                    r.kind = OptionType::Skill as u8;
                    r.area = Some(AreaType::Stadium as u8);
                }
                Action::Pass => r.kind = OptionType::End as u8,
            }
            r.player_index = Some(p);
            r
        }));
        out.player = p;
        out.select_type = SelectType::Main;
        out.context = SelectContext::Main;
        out.min_count = 1;
        out.max_count = 1;
        out.provenance = Provenance::NONE;
        out.map = Source::Turn;
    }

    pub(crate) fn prompt_select(&self, i: usize) -> SelectData {
        let mut s = SelectData::default();
        self.prompt_select_into(i, &mut s);
        s
    }

    /// Question `i` (a decision prompt) as a select, into `out` (its lists reused).
    fn prompt_select_into(&self, i: usize, out: &mut SelectData) {
        let pr = self.prompts.as_slice()[i];
        let persp = pr.player as usize;
        let ctx = context_for(pr.purpose);
        out.actions.clear();
        let SelectData { options, vals, .. } = &mut *out;
        options.clear();
        vals.clear();
        let (t, c, min, max, sh) = match pr.kind {
            PromptKind::Confirm => {
                let o = |k: OptionType| Opt { kind: k as u8, ..Default::default() };
                options.extend([o(OptionType::Yes), o(OptionType::No)]);
                vals.extend([OptionValue::Bool(true), OptionValue::Bool(false)]);
                (SelectType::YesNo, ctx, 1, 1, AnswerShape::Single)
            }
            PromptKind::Select { values, .. } => {
                let n = values.len();
                options.extend((0..n).map(|k| Opt {
                    kind: OptionType::Number as u8,
                    number: Some(match values {
                        SelectValues::DrawCards(m) => m as i32 - k as i32,
                        _ => k as i32,
                    }),
                    source: match values {
                        SelectValues::Effects(v) => v.as_slice()[k].1,
                        _ => None,
                    },
                    ..Default::default()
                }));
                vals.extend((0..n).map(OptionValue::Index));
                (SelectType::Count, ctx, 1, 1, AnswerShape::Single)
            }
            PromptKind::ChooseCards { cards, filter, opts } => {
                let list = self.prompt_list(cards);
                let (area, owner) = area_of(cards);
                for (k, &c) in list.iter().enumerate() {
                    if self.choose_cards_offers(k, c, &filter, &opts) {
                        options.push(Opt {
                            kind: OptionType::Card as u8,
                            area: Some(area),
                            index: Some(k as u8),
                            player_index: Some(owner),
                            card_id: Some(self.st.cards[c as usize].def),
                            serial: Some(c),
                            ..Default::default()
                        });
                        vals.push(OptionValue::Index(k));
                    }
                }
                let min = opts.min as usize;
                let max = (opts.max as usize).min(options.len());
                (SelectType::Card, ctx, if opts.allow_cancel { 0 } else { min }, max, AnswerShape::Array)
            }
            PromptKind::ChoosePokemon { min, max, .. } => {
                let cands = self.choose_pokemon_candidates(&pr);
                options.extend(cands.iter().map(|t| {
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
                }));
                vals.extend(cands.iter().map(|t| OptionValue::Target(*t)));
                let n = options.len();
                (SelectType::Card, ctx, (min as usize).min(n), (max as usize).min(n), AnswerShape::Array)
            }
            PromptKind::ChoosePrize { count, use_opponent_prizes, .. } => {
                let q = if use_opponent_prizes { 1 - persp } else { persp };
                let left = self.st.players[q].prizes().iter().filter(|l| !l.is_empty()).count();
                options.extend((0..left).map(|k| Opt { kind: OptionType::Card as u8, area: Some(AreaType::Prize as u8), index: Some(k as u8), player_index: Some(q as u8), ..Default::default() }));
                vals.extend((0..left).map(OptionValue::Index));
                let n = (count as usize).min(left);
                (SelectType::Card, ctx, n, n, AnswerShape::Array)
            }
            PromptKind::ChooseEnergy { energy, cost, .. } => {
                let energy = self.payload_energy(energy);
                // The options are the attached cards in the order they arrived (the selection interface's order); `index` and the wire
                // value stay the entry's position in the offered map.
                let mut order: SVec<usize, 64> = SVec::new();
                for k in 0..energy.len() {
                    order.push(k);
                }
                order.as_mut_slice().sort_by_key(|&k| self.st.cards[energy.as_slice()[k].card as usize].arrived);
                options.extend(order.iter().map(|&k| {
                    let e = &energy.as_slice()[k];
                    Opt {
                        kind: OptionType::Energy as u8,
                        area: Some(AreaType::Energy as u8),
                        index: Some(k as u8),
                        card_id: Some(self.st.cards[e.card as usize].def),
                        serial: Some(e.card),
                        ..Default::default()
                    }
                }));
                vals.extend(order.iter().map(|&k| OptionValue::Index(k)));
                let n = options.len();
                // A payment never has more cards than the cost (ruling 1652); a larger bound made the
                // pick-mask search run out of budget and offer cards that lead to a dead end.
                let max = n.min(cost.len().max(1));
                // An Energy select that puts the cards into the hand is its own context.
                let ctx = if ctx == SelectContext::ToHand { SelectContext::ToHandEnergy } else { ctx };
                (SelectType::Energy, ctx, 0, max, AnswerShape::Array)
            }
            PromptKind::AttachEnergy { cards, player_type, slots, filter, o } => {
                // One option per (energy card, target) pair; pick each energy at most once.
                let list = self.prompt_list(cards);
                let mut targets = slot_targets(&self.st, persp, player_type, slots.as_slice());
                targets.retain(|t| !o.blocked_to.iter().any(|b| b.player == t.player && b.slot == t.slot && b.index == t.index));
                for (k, &c) in list.iter().enumerate() {
                    let d = self.st.cdef(c);
                    if !(d.is_energy() && !o.blocked.contains(&(k as u8)) && filter.matches(d)) {
                        continue;
                    }
                    for t in targets.iter() {
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
                        vals.push(OptionValue::AttachTo(*t, k));
                    }
                }
                (SelectType::AttachedCard, SelectContext::AttachTo, o.min as usize, o.max as usize, AnswerShape::Array)
            }
            PromptKind::DiscardEnergy { player_type, slots, filter, o } | PromptKind::MoveEnergy { player_type, slots, filter, o } => {
                let o = *self.payload_opts(o);
                let is_move = matches!(pr.kind, PromptKind::MoveEnergy { .. });
                let all = slot_targets(&self.st, persp, player_type, slots.as_slice());
                // The offered cards (source, index in the source's list, card) of every Pokémon in the order they arrived
                // (the selection interface's order; a stable sort); each card's destinations follow it in target order.
                let mut offered: SVec<(CardTarget, u8, CardId), 128> = SVec::new();
                for (from, idx) in self.energy_sources(persp, player_type, slots.as_slice(), &filter, &o) {
                    let s = get_target(&self.st, persp, from).unwrap();
                    let slot = self.st.slot(s.p as usize, s.s);
                    for i in idx {
                        let c = if crate::prompts::is_tool_filter(&filter) { slot.tools().as_slice()[i as usize] } else { slot.cards().as_slice()[i as usize] };
                        offered.push((from, i, c));
                    }
                }
                offered.as_mut_slice().sort_by_key(|&(_, _, c)| self.st.cards[c as usize].arrived);
                for &(from, i, c) in offered.iter() {
                    let s = get_target(&self.st, persp, from).unwrap();
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
                            vals.push(OptionValue::FromTo(from, *to, i as usize));
                        }
                    } else {
                        options.push(base);
                        vals.push(OptionValue::From(from, i as usize));
                    }
                }
                let max = o.max.map(|m| m as usize).unwrap_or(options.len());
                let ctx = if is_move { SelectContext::SwitchEnergyCard } else { SelectContext::DiscardEnergyCard };
                (SelectType::AttachedCard, ctx, o.min as usize, max, AnswerShape::Array)
            }
            PromptKind::PutDamage { player_type, slots, damage, blocked, allow_partial, damage_multiple, .. } => {
                let mut targets = slot_targets(&self.st, persp, player_type, slots.as_slice());
                targets.retain(|t| !blocked.iter().any(|b| get_target(&self.st, persp, *b).ok() == get_target(&self.st, persp, *t).ok()));
                options.extend(targets.iter().map(|t| Opt {
                    kind: OptionType::Card as u8,
                    area: Some(if t.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                    index: Some(t.index),
                    player_index: Some(if t.player == PlayerType::BottomPlayer { persp as u8 } else { 1 - persp as u8 }),
                    ..Default::default()
                }));
                vals.extend(targets.iter().map(|t| OptionValue::Target(*t)));
                let n = (damage / damage_multiple.max(1)) as usize;
                (SelectType::Card, SelectContext::DamageCounter, if allow_partial { 0 } else { n }, n, AnswerShape::Counters(damage_multiple))
            }
            PromptKind::MoveDamage { player_type, slots, o, .. } | PromptKind::RemoveDamage { player_type, slots, o, .. } => {
                let o = *self.payload_opts(o);
                let all = slot_targets(&self.st, persp, player_type, slots.as_slice());
                let seat = |t: &CardTarget| if t.player == PlayerType::BottomPlayer { persp as u8 } else { 1 - persp as u8 };
                for f in all.iter() {
                    for t in all.iter().filter(|t| !(t.player == f.player && t.slot == f.slot && t.index == f.index)) {
                        options.push(Opt {
                            kind: OptionType::Card as u8,
                            area: Some(if f.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                            index: Some(f.index),
                            in_play_area: Some(if t.slot == SlotType::Active { AreaType::Active as u8 } else { AreaType::Bench as u8 }),
                            in_play_index: Some(t.index),
                            from_player: Some(seat(f)),
                            to_player: Some(seat(t)),
                            ..Default::default()
                        });
                        vals.push(OptionValue::Pair(*f, *t));
                    }
                }
                let max = o.max.map(|m| m as usize).unwrap_or(12);
                (SelectType::Card, SelectContext::DamageCounter, o.min as usize, max, AnswerShape::Array)
            }
            PromptKind::OrderCards { cards, .. } => {
                let list = self.prompt_list(cards);
                options.extend(list.iter().enumerate().map(|(k, c)| Opt { kind: OptionType::Card as u8, index: Some(k as u8), card_id: Some(self.st.cards[*c as usize].def), serial: Some(*c), ..Default::default() }));
                let n = options.len();
                vals.extend((0..n).map(OptionValue::Index));
                (SelectType::Card, SelectContext::ToDeck, n, n, AnswerShape::Array)
            }
            PromptKind::SelectOption { values, disabled, .. } => {
                for k in 0..values.len() {
                    if disabled.map(|d| d & (1 << k) != 0).unwrap_or(false) {
                        continue;
                    }
                    options.push(Opt { kind: OptionType::Number as u8, number: Some(k as i32), ..Default::default() });
                    vals.push(OptionValue::Index(k));
                }
                (SelectType::Count, ctx, 1, 1, AnswerShape::Single)
            }
            PromptKind::ChooseAttack { cards, blocked, .. } => {
                for (i, c) in cards.iter().enumerate() {
                    for (a, at) in self.st.cdef(*c).attacks.iter().enumerate() {
                        // A blocked attack is shown in the prompt's `blocked` list and is not an option.
                        if blocked.iter().any(|(bi, ba)| *bi as usize == i && *ba as usize == a) {
                            continue;
                        }
                        options.push(Opt {
                            kind: OptionType::Attack as u8,
                            attack_id: Some(a as u8),
                            card_id: Some(self.st.cards[*c as usize].def),
                            serial: Some(*c),
                            ..Default::default()
                        });
                        vals.push(OptionValue::Attack { index: i, attack: a as u8, name: at.name });
                    }
                }
                (SelectType::Attack, SelectContext::Attack, 1, 1, AnswerShape::Single)
            }
            _ => (SelectType::YesNo, SelectContext::Look, 0, 0, AnswerShape::Single),
        };
        out.player = pr.player;
        out.select_type = t;
        out.context = c;
        out.min_count = min;
        out.max_count = max;
        out.provenance = pr.provenance;
        out.map = Source::Prompt(i, sh);
    }

    /// Answer a select with option indices. Invalid combinations (e.g. an
    /// energy set that does not pay the cost exactly) return an error and
    /// leave the game unchanged.
    pub fn answer(&mut self, sel: &SelectData, chosen: &[usize]) -> R {
        self.answer_with(sel, chosen, true)
    }

    /// [`Game::answer`] for a driver whose answer is valid by construction (picked from the masks: self-play, the
    /// benchmark): no backup copy (a rejected answer leaves the game half done; `PTCG_VERIFY_LEGAL=1` asserts none is).
    pub fn answer_no_rollback(&mut self, sel: &SelectData, chosen: &[usize]) -> R {
        let r = self.answer_with(sel, chosen, false);
        if crate::options::verify_legal() {
            assert!(r.is_ok(), "an answer picked from the masks was rejected: {:?}", r);
        }
        r
    }

    /// [`Game::answer`]; `rollback` false skips the backup copy that restores the game when the answer
    /// is rejected (a caller working on a copy it throws away).
    fn answer_with(&mut self, sel: &SelectData, chosen: &[usize], rollback: bool) -> R {
        match &sel.map {
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
            Source::Turn => {
                let k = *chosen.first().ok_or(GameError("EMPTY_ANSWER"))?;
                let a = *sel.actions.get(k).ok_or(GameError("BAD_OPTION"))?;
                self.act(a)
            }
            Source::Prompt(i, _) => {
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

/// The wire answer (the form a recorded trace holds) for option indices `chosen` of a prompt select.
fn raw_answer(sel: &SelectData, chosen: &[usize]) -> Result<Value, GameError> {
    let (vals, shape) = match &sel.map {
        Source::Prompt(_, shape) => (&sel.vals, shape),
        _ => return Err(GameError("NOT_A_PROMPT")),
    };
    Ok(match shape {
                    AnswerShape::Single => vals.get(*chosen.first().ok_or(GameError("EMPTY_ANSWER"))?).map(OptionValue::json).ok_or(GameError("BAD_OPTION"))?,
                    AnswerShape::Array => {
                        let mut v = Vec::new();
                        for &k in chosen {
                            v.push(vals.get(k).map(OptionValue::json).ok_or(GameError("BAD_OPTION"))?);
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
        match &sel.map {
            Source::Prompt(i, _) => {
                let pr = self.prompts.as_slice()[*i];
                self.decode_chosen(&pr, sel, chosen).is_ok()
            }
            Source::Turn => chosen.len() == 1 && chosen[0] < sel.actions.len(),
            Source::Chance(_) => chosen.len() == 1,
        }
    }

    /// Whether some superset of `chosen` could still pass decode + validate.
    /// For the prompt kinds whose checks (other than the minimum count) only
    /// get stricter as picks are added, this is validity with the minimum
    /// dropped; other kinds are assumed extendable.
    fn extendable(&self, sel: &SelectData, chosen: &[usize]) -> bool {
        let i = match &sel.map {
            Source::Prompt(i, _) => *i,
            _ => return true,
        };
        let mut pr = self.prompts.as_slice()[i];
        match pr.kind {
            PromptKind::ChooseCards { .. }
            | PromptKind::ChoosePokemon { .. }
            | PromptKind::AttachEnergy { .. }
            | PromptKind::DiscardEnergy { .. }
            | PromptKind::MoveEnergy { .. }
            | PromptKind::MoveDamage { .. }
            | PromptKind::RemoveDamage { .. }
            | PromptKind::PutDamage { .. } => {}
            PromptKind::ChooseEnergy { cost, .. } => {
                // Payments never have more cards than the cost (ruling 1652): longer picks cannot be completed.
                return sel.picks_ok(chosen) && chosen.len() <= cost.len();
            }
            _ => return true,
        }
        // The decode of a partial answer (the declared way to drop the minimum count, `PromptRec::partial`).
        pr.partial = true;
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
        let Source::Prompt(i, _) = &sel.map else { return Err(GameError("NOT_A_PROMPT")) };
        let pr = self.prompts.as_slice()[*i];
        self.decode_chosen(&pr, sel, chosen)
    }

    /// The answer for option indices `chosen`: the picked options' values read by the prompt kind's decoder, no JSON
    /// built. `PTCG_VERIFY_CACHE=1` asserts the wire form (`raw_answer`) read back (`decode_answer`) gives the same.
    fn decode_chosen(&self, pr: &PromptRec, sel: &SelectData, chosen: &[usize]) -> Result<Res, GameError> {
        let r = self.decode_chosen_typed(pr, sel, chosen);
        if crate::game::verify_cache() {
            let json = raw_answer(sel, chosen).and_then(|raw| self.decode_answer(pr, &raw));
            assert_eq!(format!("{:?}", r), format!("{:?}", json), "typed decode differs from the JSON decode of {:?}", chosen);
        }
        r
    }

    fn decode_chosen_typed(&self, pr: &PromptRec, sel: &SelectData, chosen: &[usize]) -> Result<Res, GameError> {
        let Source::Prompt(_, shape) = &sel.map else { return Err(GameError("NOT_A_PROMPT")) };
        let vals = &sel.vals;
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
            OptionValue::Index(i) => Some(*i),
            _ => None,
        };
        let target = |k: &usize| match &vals[*k] {
            OptionValue::Target(t) => Some(*t),
            _ => None,
        };
        match (&pr.kind, shape) {
            (PromptKind::ChooseCards { cards, filter, opts }, AnswerShape::Array) => self.decode_choose_cards(*cards, filter, opts, pr.partial, chosen.iter().map(idx)),
            (PromptKind::ChoosePokemon { .. }, AnswerShape::Array) => self.decode_choose_pokemon(pr, chosen.iter().map(target)),
            (PromptKind::ChoosePrize { .. }, AnswerShape::Array) => self.decode_choose_prize(pr, chosen.iter().map(idx)),
            (PromptKind::ChooseEnergy { energy, cost, .. }, AnswerShape::Array) => self.decode_choose_energy(self.payload_energy(*energy), cost, chosen.iter().map(idx)),
            (PromptKind::AttachEnergy { .. }, AnswerShape::Array) if chosen.len() <= Self::extra_cap(&pr.kind) => self.decode_attach(
                pr,
                chosen.iter().map(|k| match &vals[*k] {
                    OptionValue::AttachTo(t, i) => Some((*t, *i)),
                    _ => None,
                }),
            ),
            (PromptKind::DiscardEnergy { .. }, AnswerShape::Array) if chosen.len() <= Self::extra_cap(&pr.kind) => self.decode_discard_energy(
                pr,
                chosen.iter().map(|k| match &vals[*k] {
                    OptionValue::From(f, i) => Some((*f, *i)),
                    _ => None,
                }),
            ),
            (PromptKind::MoveEnergy { .. }, AnswerShape::Array) if chosen.len() <= Self::extra_cap(&pr.kind) => self.decode_move_energy(
                pr,
                chosen.iter().map(|k| match &vals[*k] {
                    OptionValue::FromTo(f, t, i) => Some((*f, *t, *i)),
                    _ => None,
                }),
            ),
            (PromptKind::MoveDamage { .. } | PromptKind::RemoveDamage { .. }, AnswerShape::Array) => self.decode_move_damage(
                pr,
                chosen.iter().map(|k| match &vals[*k] {
                    OptionValue::Pair(f, t) => Some((*f, *t)),
                    _ => None,
                }),
            ),
            (PromptKind::OrderCards { .. }, AnswerShape::Array) if chosen.len() <= Self::extra_cap(&pr.kind) => self.decode_order(pr, chosen.iter().map(idx)),
            (PromptKind::PutDamage { .. }, AnswerShape::Counters(mult)) => {
                // Picks of one target add up (`raw_answer`).
                let mut agg: SVec<(usize, i32), 32> = SVec::new();
                for &k in chosen {
                    match agg.as_mut_slice().iter_mut().find(|x| x.0 == k) {
                        Some(x) => x.1 += *mult,
                        None => agg.push((k, *mult)),
                    }
                }
                if agg.len() > Self::extra_cap(&pr.kind) {
                    return Err(GameError("INVALID_PROMPT_RESULT"));
                }
                self.decode_put_damage(
                    pr,
                    agg.iter().map(|(k, d)| match &vals[*k] {
                        OptionValue::Target(t) => Some((*t, *d)),
                        _ => None,
                    }),
                )
            }
            (PromptKind::Confirm, AnswerShape::Single) => match &vals[chosen[0]] {
                OptionValue::Bool(b) => Ok(Res::Bool(*b)),
                _ => Err(GameError("INVALID_PROMPT_RESULT")),
            },
            (PromptKind::Select { .. } | PromptKind::SelectOption { .. }, AnswerShape::Single) => match &vals[chosen[0]] {
                OptionValue::Index(i) => Ok(Res::Int(*i as i32)),
                _ => Err(GameError("INVALID_PROMPT_RESULT")),
            },
            (PromptKind::ChooseAttack { .. }, AnswerShape::Single) => match &vals[chosen[0]] {
                OptionValue::Attack { index, attack, .. } => self.decode_choose_attack(pr, *index, *attack as usize),
                _ => Err(GameError("INVALID_PROMPT_RESULT")),
            },
            // Longer than the prompt kind takes (`extra_cap`); no other pair of a prompt kind and an answer shape is
            // built (`prompt_select`).
            _ => Err(GameError("INVALID_PROMPT_RESULT")),
        }
    }

    /// For a pick-by-pick answer: which options can be picked next so that a valid answer is still reachable, and
    /// whether `picks` can be submitted as is (STOP). Bounded search; if the bound is hit an option is assumed reachable.
    ///
    /// Declared validity (events batch 8 C9): an answer is accepted when the prompt's decode accepts it. The decode reads
    /// what the prompt declares (its cards and filter, the blocked sources, destinations and cards, the counts, the
    /// cost), the same restrictions its continuation checks, so no answer is resolved on a copy of the game.
    /// `PTCG_VERIFY_LEGAL=1` resolves every option and every completion found on a copy and asserts the continuation
    /// accepts exactly the declared answers.
    pub fn pick_mask(&self, sel: &SelectData, picks: &[usize]) -> (Vec<bool>, bool) {
        let mut mask = Vec::new();
        let stop = self.pick_mask_into(sel, picks, &mut mask);
        (mask, stop)
    }

    /// [`Game::pick_mask`] into a caller's buffer (no allocation once it has the room); returns STOP.
    pub fn pick_mask_into(&self, sel: &SelectData, picks: &[usize], mask: &mut Vec<bool>) -> bool {
        let n = sel.options.len();
        mask.clear();
        if !matches!(sel.map, Source::Prompt(..)) {
            mask.resize(n, true);
            return false;
        }
        if crate::options::verify_legal() {
            for j in 0..n {
                self.verify_does_something(sel, j);
            }
            // Every question asked can be answered (`Game::prompt` doesn't ask one that can't).
            if let (true, Source::Prompt(i, _)) = (picks.is_empty(), &sel.map) {
                assert!(self.has_an_answer(*i), "a question with no answer it can take is asked: {:?}", self.describe_prompt(&self.prompts.as_slice()[*i]));
            }
        }
        if sel.max_count <= 1 {
            // Single pick: the options the prompt accepts.
            mask.extend((0..n).map(|j| self.is_declared_answer(sel, &[j])));
            return sel.min_count == 0 && self.is_declared_answer(sel, &[]);
        }
        let repeats = sel.allows_repeats();
        let exact = match &sel.map {
            Source::Prompt(i, _) => matches!(self.prompts.as_slice()[*i].kind, PromptKind::ChooseEnergy { .. }),
            _ => false,
        };
        let stop = picks.len() >= sel.min_count && self.is_declared_answer(sel, picks);
        mask.resize(n, false);
        if picks.len() >= sel.max_count {
            return stop;
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
            mask[j] = self.completable(sel, &mut buf, 0, repeats, &mut budget, true);
            buf.pop();
        }
        stop
    }

    /// The prompt accepts `chosen` (its decode does: declared validity). `PTCG_VERIFY_LEGAL=1`: resolving it on a copy
    /// agrees.
    fn is_declared_answer(&self, sel: &SelectData, chosen: &[usize]) -> bool {
        let ok = self.is_valid_answer(sel, chosen);
        if crate::options::verify_legal() {
            let mut trial = self.fork();
            let accepted = ok && trial.answer_with(sel, chosen, false).is_ok();
            assert_eq!(ok, accepted, "declared validity differs from the continuation for {:?} of {:?}", chosen, self.describe_prompt(&self.prompts.as_slice()[match &sel.map {
                Source::Prompt(i, _) => *i,
                _ => 0,
            }]));
        }
        ok
    }

    /// Depth-first search for a valid superset of `buf`; the added options are
    /// enumerated in index order from `start` (combinations, not permutations). `declared`: an answer found is checked as
    /// [`Game::is_declared_answer`] does (`PTCG_VERIFY_LEGAL` resolves it on a copy); otherwise it is only decoded.
    fn completable(&self, sel: &SelectData, buf: &mut Vec<usize>, start: usize, repeats: bool, budget: &mut usize, declared: bool) -> bool {
        if *budget == 0 {
            return true;
        }
        *budget -= 1;
        if !self.extendable(sel, buf) {
            return false;
        }
        let valid = |g: &Game, buf: &[usize]| if declared { g.is_declared_answer(sel, buf) } else { g.is_valid_answer(sel, buf) };
        if buf.len() >= sel.min_count && valid(self, buf) {
            return true;
        }
        if buf.len() >= sel.max_count {
            return false;
        }
        for k in start..sel.options.len() {
            if !repeats && buf.contains(&k) {
                continue;
            }
            buf.push(k);
            let ok = self.completable(sel, buf, if repeats { k } else { k + 1 }, repeats, budget, declared);
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

    /// Can the question `i` (a decision prompt) be answered: is some answer valid, a cancel or no choice included? A
    /// question with no answer it can take isn't asked (`Game::prompt`): e.g. an evolution target when every Pokémon is
    /// blocked, an attach with no Pokémon to attach to. Answers are only decoded (the program that opens the question is
    /// still running: nothing is resolved on a copy); the search budget, when hit, counts as an answer.
    pub(crate) fn has_an_answer(&self, i: usize) -> bool {
        let pr = self.prompts.as_slice()[i];
        // A prompt whose null answer is accepted (a cancel; a yes / no or a number with no validation) can be answered.
        if self.decline_answer(&pr).is_ok() {
            return true;
        }
        let sel = self.prompt_select(i);
        if sel.min_count == 0 && self.is_valid_answer(&sel, &[]) {
            return true;
        }
        let mut buf = Vec::new();
        let mut budget = 256usize;
        self.completable(&sel, &mut buf, 0, sel.allows_repeats(), &mut budget, false)
    }
}

impl Game {
    /// `PTCG_VERIFY_LEGAL=1` (events batch 8 C10, docs/rulings/RULES.md "Options that can't do anything"): an option
    /// offered as an answer on its own (not a "no" nor a cancel) changes the game, or leads to a choice, a shuffle, a
    /// flip or a reveal; only an option whose effect a prevention stops may do nothing (a protected Pokémon stays
    /// choosable: id290; JP FAQ Hariyama MEG 73).
    fn verify_does_something(&self, sel: &SelectData, j: usize) {
        let Source::Prompt(i, _) = &sel.map else { return };
        let vals = &sel.vals;
        if matches!(vals.get(j), Some(OptionValue::Bool(false))) || !self.is_valid_answer(sel, &[j]) {
            return;
        }
        let mut trial = self.fork();
        if trial.answer_with(sel, &[j], false).is_err() {
            return;
        }
        let same = crate::derived::state_digest(&self.st) == crate::derived::state_digest(&trial.st);
        let settled = matches!(trial.pending(), crate::game::Pending::Turn(_) | crate::game::Pending::Finished | crate::game::Pending::Stuck);
        let prevented = trial.prevented_events != self.prevented_events;
        assert!(
            !(same && settled && !prevented),
            "option {} of {:?} offered but it can't change the game",
            j,
            self.describe_prompt(&self.prompts.as_slice()[*i])
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The options of a damage counter move name the player of both ends, not only the spot (a move between the two sides
    /// is told from one within a side); the observation features of the option are the same as without them.
    #[test]
    fn move_damage_options_name_both_players() {
        let names: Vec<&str> = (0..60).map(|i| if i < 4 { "Duraludon PRE 69" } else { "Metal Energy MEE 8" }).collect();
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(3);
        g.start([&deck, &deck]).unwrap();
        g.settle().unwrap();
        let sc = serde_json::json!({"me": {"reset": true, "active": "Duraludon PRE 69", "bench": [{"card": "Duraludon PRE 69"}]}, "opp": {"reset": true, "active": "Duraludon PRE 69"}});
        crate::scenario::apply(&mut g, &sc).unwrap();
        let me = 0usize;
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let o = g.stash_opts(crate::prompts::MoveOpts::default());
        let kind = PromptKind::MoveDamage { player_type: PlayerType::Any, slots, max_allowed: SVec::new(), o, single_source: false, single_destination: false, damage_multiple: 10 };
        g.prompt(me, crate::purpose::PromptPurpose::MoveDamage, kind, Provenance::NONE, crate::game::Cont::Noop);
        let sel = g.prompt_select(g.prompts.len() - 1);
        // My Active, my Bench 0 and the opponent's Active: six ordered pairs.
        assert_eq!(sel.options.len(), 6);
        let pair = |o: &Opt| (o.area, o.index, o.from_player, o.in_play_area, o.in_play_index, o.to_player);
        let a = AreaType::Active as u8;
        let b = AreaType::Bench as u8;
        assert!(sel.options.iter().any(|o| pair(o) == (Some(a), Some(0), Some(0), Some(b), Some(0), Some(0))), "within my side");
        assert!(sel.options.iter().any(|o| pair(o) == (Some(a), Some(0), Some(0), Some(a), Some(0), Some(1))), "to the opponent's Active");
        assert!(sel.options.iter().any(|o| pair(o) == (Some(a), Some(0), Some(1), Some(a), Some(0), Some(0))), "from the opponent's Active");
        assert!(sel.options.iter().all(|o| o.from_player.is_some() && o.to_player.is_some() && o.player_index.is_none()));
        // Each option's typed value is its part of the wire answer; the question can be declined (a cancel).
        for j in 0..sel.options.len() {
            let v = sel.option_value(j).unwrap();
            assert!(matches!(v, OptionValue::Pair(..)));
            assert_eq!(sel.wire_answer(&[j]).unwrap(), Value::Array(vec![v.json()]));
        }
        assert_eq!(sel.option_value(6), None);
        assert!(matches!(g.decline_answer(&g.prompts.as_slice()[g.prompts.len() - 1]), Ok(Res::Null)));
    }
}
