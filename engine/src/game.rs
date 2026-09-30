//! The store: effect propagation, prompts with continuations, dispatch.
//!
//! A faithful port of Twinleaf's `Store`. Continuations are data
//! ([`Cont`]) held in the state, so a game with pending prompts can be cloned
//! and resumed. Every Twinleaf generator is an explicit frame that is moved
//! into the continuation of the prompt it yields on.

use crate::cards::{self, CardFrame};
use crate::effects::*;
use crate::engine::{attack, check, phase, play, retreat, setup, turn};
use crate::list::*;
use crate::prompts::*;
use crate::rng::Rng;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameError(pub &'static str);

pub type R<T = ()> = Result<T, GameError>;

#[macro_export]
macro_rules! bail {
    ($m:expr) => {
        return Err($crate::game::GameError($m))
    };
}

/// Continuation run when a prompt group resolves (or a wait item fires).
#[derive(Clone, Copy, Debug)]
pub enum Cont {
    Noop,
    Setup(setup::SetupFrame),
    /// WaitPrompt after the turn draw: `state.phase = PLAYER_TURN`.
    PhasePlayerTurn,
    CheckState(check::CheckFrame),
    TakePrizes { p: u8, destination: ListRef },
    ChooseActive { p: u8 },
    /// handleBenchSizeChange discard prompt: `empty` = bitmask of empty bench slot ids.
    BenchShrink { p: u8, empty: u16 },
    BetweenTurnsWait { oc: OnComplete },
    BetweenTurnsCheck { oc: OnComplete },
    BurnFlip { p: u8, slot: SlotId },
    SleepFlips { p: u8, slot: SlotId },
    UseAttack(attack::AttackFrame),
    UsePower(attack::PowerFrame),
    Retreat(retreat::RetreatCont),
    /// Coin flip animation wait; then the flip callback runs with `result`.
    CoinFlipWait { cb: CoinCb, result: bool },
    TrainerCleanup { p: u8, card: CardId },
    ShuffleApply { p: u8 },
    Prefab(crate::prefabs::PrefabCont),
    Card { card: CardId, frame: CardFrame },
    /// A card frame created by `imp`'s handler running for `card` (copy-attack delegation).
    CardAs { imp: CardId, card: CardId, frame: CardFrame },
    /// COPY_ATTACK_FROM_POKEMON_LIST / runDelegatedCopiedAttackGenerator.
    CopyAttack(crate::copy_attack::CopyFrame),
}

/// `checkState(..., onComplete)` callbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnComplete {
    None,
    /// EndTurn: expire KO-time effects, then `startNextTurn`.
    AfterEndTurn { p: u8 },
    /// betweenTurns completion: `initNextTurn` unless finished.
    InitNextTurn,
}

/// Coin flip callbacks (`CoinFlipEffect.callback`).
#[derive(Clone, Copy, Debug)]
pub enum CoinCb {
    None,
    Card { card: CardId, frame: CardFrame },
    Attack(attack::AttackCoinCb),
    /// RUN_COIN_FLIP_SEQUENCE: one flip done; `results` bit i = flip i heads.
    Sequence { p: u8, mode: u8, results: u32, n: u8, callback: u8 },
    /// Final callback of a sequence: card receives (bitmask, count) via `frame.a[2..]`.
    SequenceCard { card: CardId, frame: CardFrame },
}

#[derive(Clone, Copy, Debug)]
pub struct PromptItem {
    pub ids: SVec<u32, 4>,
    pub cont: Cont,
}

#[derive(Clone, Copy, Debug)]
pub struct EffSlot {
    pub e: Effect,
    pub prevent_default: bool,
    pub refs: u8,
}

pub const MAX_FX: usize = 24;

/// Player actions (`game-actions.ts`, `play-card-action.ts`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    PlayCard { hand_index: u8, target: CardTarget },
    Attack { name: &'static str },
    UseAbility { name: &'static str, target: CardTarget },
    UseTrainerAbility { name: &'static str, target: CardTarget },
    UseStadium,
    Retreat { bench_index: u8 },
    Pass,
}

/// What the game is waiting for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pending {
    Finished,
    /// Unresolved chance prompt (index into `prompts`).
    Chance(usize),
    /// Unresolved info prompt.
    Info(usize),
    /// Unresolved decision prompt.
    Decision(usize),
    /// Turn action for this player index.
    Turn(u8),
    Stuck,
}

#[derive(Clone, Copy)]
pub struct Game {
    pub st: State,
    pub rng: Rng,
    pub prompts: SVec<PromptRec, 10>,
    pub last_prompt_id: u32,
    pub items: SVec<PromptItem, 10>,
    pub waits: SVec<Cont, 16>,
    pub fx: SVec<EffSlot, MAX_FX>,
    pub temps: [List<60>; 3],
    pub temp_used: [bool; 3],
    /// Callbacks referenced by pending `CoinFlipEffect`s.
    pub coin_callbacks: SVec<CoinCb, 16>,
    /// Trainer whose effect is creating prompts (`Store.resolvingTrainer`).
    pub resolving_trainer: Option<(u8, CardId)>,
    /// `probingStadiumEffect` (stadium-effect.ts module state).
    pub probing_stadium: bool,
    /// Union of subscription masks of every card in the game (skip propagation otherwise).
    pub kinds_present: crate::effects::KindMask,
    /// Opt-in effect-type trace for the diff tool (not part of rules state).
    pub trace_effects: bool,
    /// `copyAttackSessions` (copy-attack-delegation.ts module state).
    pub copy_sessions: SVec<crate::copy_attack::CopySession, 8>,
    pub copy_gen: u8,
    pub copy_serial: u16,
    /// Source handler currently running bound to a copycat.
    pub delegating: Option<crate::copy_attack::Delegating>,
}

pub struct EffectLog;

thread_local! {
    pub static EFFECT_TRACE: std::cell::RefCell<Vec<&'static str>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl Game {
    pub fn new(seed: u32) -> Game {
        Game {
            st: State::new(),
            rng: Rng::new(seed),
            prompts: SVec::new(),
            last_prompt_id: 0,
            items: SVec::new(),
            waits: SVec::new(),
            fx: SVec::new(),
            temps: [List::new(); 3],
            temp_used: [false; 3],
            coin_callbacks: SVec::new(),
            resolving_trainer: None,
            probing_stadium: false,
            kinds_present: 0,
            trace_effects: false,
            copy_sessions: SVec::new(),
            copy_gen: 0,
            copy_serial: 0,
            delegating: None,
        }
    }

    // -----------------------------------------------------------------------
    // Effect arena

    pub fn new_fx(&mut self, e: Effect) -> EffId {
        self.fx.push(EffSlot { e, prevent_default: false, refs: 1 });
        (self.fx.len() - 1) as EffId
    }

    pub fn retain_fx(&mut self, id: EffId) {
        self.fx.as_mut_slice()[id as usize].refs += 1;
    }

    pub fn release_fx(&mut self, id: EffId) {
        let s = &mut self.fx.as_mut_slice()[id as usize];
        s.refs = s.refs.saturating_sub(1);
        while let Some(top) = self.fx.as_slice().last() {
            if top.refs == 0 {
                self.fx.pop();
            } else {
                break;
            }
        }
    }

    #[inline]
    pub fn e(&self, id: EffId) -> &Effect {
        &self.fx.as_slice()[id as usize].e
    }
    #[inline]
    pub fn e_mut(&mut self, id: EffId) -> &mut Effect {
        &mut self.fx.as_mut_slice()[id as usize].e
    }
    #[inline]
    pub fn prevented(&self, id: EffId) -> bool {
        self.fx.as_slice()[id as usize].prevent_default
    }
    pub fn set_prevent(&mut self, id: EffId, v: bool) {
        self.fx.as_mut_slice()[id as usize].prevent_default = v;
    }

    /// `new CoinFlipEffect(player, callback)` reduced (COIN_FLIP_PROMPT).
    pub fn coin_flip(&mut self, p: usize, cb: CoinCb) -> R<Option<bool>> {
        self.coin_callbacks.push(cb);
        let k = (self.coin_callbacks.len() - 1) as u8;
        let (e, _) = self.run_fx(Effect::CoinFlip { p: p as u8, callback: Some(k), result: None, skip_reflip_stadium: false, skip_reflip_tool: false })?;
        Ok(match e {
            Effect::CoinFlip { result, .. } => result,
            _ => None,
        })
    }

    /// Create, reduce and release an effect; returns its final value and
    /// whether it was prevented. Errors propagate like a thrown GameError.
    pub fn run_fx(&mut self, e: Effect) -> R<(Effect, bool)> {
        let id = self.new_fx(e);
        let r = self.reduce_effect(id);
        let out = (*self.e(id), self.prevented(id));
        self.release_fx(id);
        r.map(|_| out)
    }

    // -----------------------------------------------------------------------
    // Temp lists

    pub fn alloc_temp(&mut self, cards: &[CardId]) -> ListRef {
        for i in 0..self.temps.len() {
            if !self.temp_used[i] {
                self.temp_used[i] = true;
                self.temps[i].set_from(cards);
                return ListRef::Temp(i as u8);
            }
        }
        panic!("temp lists exhausted");
    }

    // -----------------------------------------------------------------------
    // Prompts

    pub fn has_prompts(&self) -> bool {
        !self.items.is_empty()
    }

    fn next_id(&mut self) -> u32 {
        self.last_prompt_id += 1;
        self.last_prompt_id
    }

    /// `store.prompt(state, prompt, then)`.
    pub fn prompt(&mut self, player_id: u8, message: &'static str, kind: PromptKind, cont: Cont) {
        let cont = crate::copy_attack::route_cont(self, cont);
        let id = self.next_id();
        self.prompts.push(PromptRec { id, player_id, perspective: None, message, kind, result: None, trainer: self.resolving_trainer });
        let mut ids = SVec::new();
        ids.push(id);
        self.items.push(PromptItem { ids, cont });
    }

    /// `store.prompt(state, [prompts...], then)`.
    pub fn prompt_group(&mut self, prompts: &[(u8, &'static str, PromptKind)], cont: Cont) {
        let cont = crate::copy_attack::route_cont(self, cont);
        let mut ids = SVec::new();
        for &(player_id, message, kind) in prompts {
            let id = self.next_id();
            self.prompts.push(PromptRec { id, player_id, perspective: None, message, kind, result: None, trainer: self.resolving_trainer });
            ids.push(id);
        }
        self.items.push(PromptItem { ids, cont });
    }

    pub fn wait_prompt(&mut self, cont: Cont) {
        let cont = crate::copy_attack::route_cont(self, cont);
        self.waits.push(cont);
    }

    /// `WaitPrompt` with a continuation.
    pub fn wait(&mut self, player_id: u8, cont: Cont) {
        self.prompt(player_id, "", PromptKind::Wait, cont);
    }

    fn resolve_wait_items(&mut self) -> R {
        while self.items.is_empty() && !self.waits.is_empty() {
            let c = self.waits.pop().unwrap();
            self.run_cont(c, &[])?;
        }
        Ok(())
    }

    /// `Store.reducePrompt`.
    fn reduce_prompt(&mut self, id: u32, result: Res) -> R {
        let pi = match self.prompts.iter().position(|p| p.id == id) {
            Some(i) => i,
            None => return Ok(()),
        };
        let ii = match self.items.iter().position(|it| it.ids.contains(&id)) {
            Some(i) => i,
            None => return Ok(()),
        };
        if self.prompts.as_slice()[pi].result.is_some() {
            bail!("PROMPT_ALREADY_RESOLVED");
        }
        self.prompts.as_mut_slice()[pi].result = Some(result);
        let item = self.items.as_slice()[ii];
        let all = item.ids.iter().all(|pid| {
            self.prompts.iter().find(|p| p.id == *pid).map(|p| p.result.is_some()).unwrap_or(false)
        });
        let r: R = (|| {
            if all {
                let mut results: SVec<Res, 4> = SVec::new();
                let mut source = None;
                for pid in item.ids.iter() {
                    if let Some(p) = self.prompts.iter().find(|p| p.id == *pid).copied() {
                        let r = self.filter_trainer_result(&p, p.result.unwrap())?;
                        results.push(r);
                        if p.trainer.is_some() {
                            source = p.trainer;
                        }
                    }
                }
                // Resolved prompts are no longer needed.
                self.prompts.retain(|p| !item.ids.contains(&p.id));
                // Twinleaf runs `then` while the item is still queued and
                // splices it afterwards at the index computed beforehand.
                let prev = self.resolving_trainer;
                if source.is_some() {
                    self.resolving_trainer = source;
                }
                let rc = self.run_cont(item.cont, results.as_slice());
                self.resolving_trainer = prev;
                rc?;
                self.items.remove_at(ii);
            }
            self.resolve_wait_items()
        })();
        if r.is_err() {
            self.prompts.retain(|p| !item.ids.contains(&p.id));
            if let Some(idx) = self.items.iter().position(|it| it.ids.contains(&id)) {
                self.items.remove_at(idx);
            }
        }
        r
    }

    fn run_cont(&mut self, c: Cont, results: &[Res]) -> R {
        let first = results.first().copied().unwrap_or(Res::Null);
        match c {
            Cont::Noop => Ok(()),
            Cont::Setup(f) => setup::resume(self, f, results),
            Cont::PhasePlayerTurn => {
                self.st.phase = GamePhase::PlayerTurn;
                Ok(())
            }
            Cont::CheckState(f) => check::resume(self, f),
            Cont::TakePrizes { p, destination } => check::take_prizes_cont(self, p, destination, first),
            Cont::ChooseActive { p } => check::choose_active_cont(self, p, first),
            Cont::BenchShrink { p, empty } => check::bench_shrink_cont(self, p, empty, first),
            Cont::BetweenTurnsWait { oc } => phase::run_between_turns_effects(self, oc),
            Cont::BetweenTurnsCheck { oc } => check::check_state(self, oc),
            Cont::BurnFlip { p, .. } => {
                if first.as_bool() {
                    phase::remove_active_condition(self, p as usize, SpecialCondition::Burned);
                }
                Ok(())
            }
            Cont::SleepFlips { p, .. } => {
                if results.iter().all(|r| r.as_bool()) {
                    phase::remove_active_condition(self, p as usize, SpecialCondition::Asleep);
                }
                Ok(())
            }
            Cont::UseAttack(f) => attack::resume_use_attack(self, f, first),
            Cont::UsePower(f) => attack::resume_use_power(self, f),
            Cont::Retreat(rc) => retreat::resume(self, rc, first),
            Cont::CoinFlipWait { cb, result } => self.run_coin_cb(cb, result),
            Cont::TrainerCleanup { p, card } => {
                play::trainer_cleanup(self, p as usize, card);
                Ok(())
            }
            Cont::ShuffleApply { p } => {
                if let Res::Order(o) = first {
                    apply_order(&mut self.st.players[p as usize].deck, o.as_slice());
                }
                // SHUFFLE_DECK's silent wait prompt.
                let id = self.st.players[p as usize].id;
                self.wait(id, Cont::Noop);
                Ok(())
            }
            Cont::Prefab(c) => crate::prefabs::resume(self, c, results),
            Cont::Card { card, frame } => cards::resume(self, card, frame, results),
            Cont::CardAs { imp, card, frame } => crate::copy_attack::resume_card_as(self, imp, card, frame, results),
            Cont::CopyAttack(f) => crate::copy_attack::resume(self, f, results),
        }
    }

    pub fn run_coin_cb(&mut self, cb: CoinCb, result: bool) -> R {
        match cb {
            CoinCb::None => Ok(()),
            CoinCb::Card { card, frame } => cards::coin_result(self, card, frame, result),
            CoinCb::Attack(a) => attack::coin_cb(self, a, result),
            CoinCb::Sequence { p, mode, results, n, callback } => {
                let results = if result { results | (1 << n) } else { results };
                let n = n + 1;
                let more = if mode == 0 { result } else { n < mode };
                if more {
                    let cb = CoinCb::Sequence { p, mode, results, n, callback };
                    self.coin_callbacks.push(cb);
                    let k = (self.coin_callbacks.len() - 1) as u8;
                    self.run_fx(Effect::CoinFlip { p, callback: Some(k), result: None, skip_reflip_stadium: true, skip_reflip_tool: true })?;
                    return Ok(());
                }
                // Reflip offers (stadium/tool) are applied by those cards' handlers.
                let fin = self.coin_callbacks.as_slice()[callback as usize];
                match fin {
                    CoinCb::SequenceCard { card, mut frame } => {
                        frame.a[2] = results as i32;
                        frame.a[3] = n as i32;
                        cards::resume(self, card, frame, &[])
                    }
                    other => self.run_coin_cb(other, result),
                }
            }
            CoinCb::SequenceCard { card, frame } => cards::resume(self, card, frame, &[]),
        }
    }

    // -----------------------------------------------------------------------
    // Effect propagation

    /// Cards with a handler for this effect kind, in Twinleaf's
    /// `propagateEffect` order (zone order, then stable sort by rank).
    fn propagation_order(&self, e: &Effect, kind: u32) -> SVec<CardId, 120> {
        let mut cards: SVec<CardId, 120> = SVec::new();
        let bit = 1u128 << kind;
        let add = |c: CardId, cards: &mut SVec<CardId, 120>| {
            if let Some(imp) = cards::impl_for(self.st.cards[c as usize].def) {
                if imp.mask & bit != 0 {
                    cards.push(c);
                }
            }
        };
        for p in 0..2 {
            let pl = &self.st.players[p];
            for c in pl.stadium.iter() {
                add(c, &mut cards);
            }
            for c in pl.supporter.iter() {
                add(c, &mut cards);
            }
            let a = &pl.slots[pl.active as usize];
            for c in a.cards.iter() {
                add(c, &mut cards);
            }
            for c in a.tools.iter() {
                add(c, &mut cards);
            }
            for &b in pl.bench.iter() {
                let s = &pl.slots[b as usize];
                for c in s.cards.iter() {
                    add(c, &mut cards);
                }
                for c in s.tools.iter() {
                    add(c, &mut cards);
                }
            }
            for i in 0..pl.prize_count as usize {
                for c in pl.prizes[i].iter() {
                    add(c, &mut cards);
                }
            }
            for c in pl.hand.iter() {
                add(c, &mut cards);
            }
            for c in pl.deck.iter() {
                add(c, &mut cards);
            }
            for c in pl.discard.iter() {
                add(c, &mut cards);
            }
        }
        if cards.len() < 2 {
            return cards;
        }
        let rank = |c: CardId| -> u8 {
            let d = self.st.cdef(c);
            match e {
                Effect::Power { .. } => match d.super_type {
                    2 => 0,
                    3 => 1,
                    _ => 2,
                },
                Effect::CheckPokemonPowers { .. } => match d.super_type {
                    1 => 0,
                    3 => 1,
                    2 if d.trainer_type == TrainerType::Stadium as u8 => 3,
                    _ => 2,
                },
                _ => d.super_type,
            }
        };
        cards.as_mut_slice().sort_by_key(|c| rank(*c)); // stable
        cards
    }

    /// `Store.reduceEffect`.
    pub fn reduce_effect(&mut self, id: EffId) -> R {
        if self.trace_effects {
            let t = self.e(id).type_name();
            EFFECT_TRACE.with(|v| v.borrow_mut().push(t));
        }
        // Ability-lock activation order bookkeeping.
        match *self.e(id) {
            Effect::MovedToActive { p, card } => {
                let slot = self.st.players[p as usize].active;
                crate::engine::game_effect::stamp_ability_lock_activation(self, p as usize, slot, card);
            }
            Effect::MovedFromActiveToBench { card, .. } => {
                crate::engine::game_effect::clear_ability_lock_activation(self, card);
            }
            _ => {}
        }

        // Propagate to cards (PlayPokemonEffect: target's tools first).
        let e = *self.e(id);
        let kind = e.kind();
        let mut first: SVec<CardId, 4> = SVec::new();
        if let Effect::PlayPokemon { target, .. } = e {
            for c in self.st.slot(target.p as usize, target.s).tools.iter() {
                first.push(c);
            }
        }
        for &c in first.iter() {
            self.call_card(c, id, kind)?;
        }
        let order = if self.kinds_present & (1u128 << kind) != 0 { self.propagation_order(&e, kind) } else { SVec::new() };
        for &c in order.iter() {
            if first.contains(&c) {
                continue;
            }
            self.call_card(c, id, kind)?;
        }
        crate::copy_attack::resolve_sessions(self, id)?;

        if self.prevented(id) {
            return Ok(());
        }

        phase::reducer(self, id)?;
        play::play_energy_reducer(self, id)?;
        play::play_pokemon_reducer(self, id)?;
        play::play_pokemon_from_zone_reducer(self, id)?;
        play::play_trainer_reducer(self, id)?;
        retreat::reducer(self, id)?;
        crate::engine::game_effect::reducer(self, id)?;
        attack::reducer(self, id)?;
        check::check_state_reducer(self, id)?;
        Ok(())
    }

    #[inline]
    fn call_card(&mut self, c: CardId, id: EffId, kind: u32) -> R {
        let d = self.st.cards[c as usize].def;
        if let Some(imp) = cards::impl_for(d) {
            if imp.mask & (1u128 << kind) != 0 {
                if let Effect::Trainer { p, card, .. } = *self.e(id) {
                    if card == c {
                        let prev = self.resolving_trainer;
                        self.resolving_trainer = Some((p, c));
                        let r = (imp.reduce)(self, c, id);
                        self.resolving_trainer = prev;
                        return r;
                    }
                }
                return (imp.reduce)(self, c, id);
            }
        }
        Ok(())
    }

    /// `filterTrainerPromptResult`: drop opponent slots whose TrainerTargetEffect is blocked.
    fn filter_trainer_result(&mut self, pr: &PromptRec, res: Res) -> R<Res> {
        let (tp, tc) = match pr.trainer {
            Some(t) => t,
            None => return Ok(res),
        };
        if let (PromptKind::ChoosePokemon { .. }, Res::Slots(sel)) = (pr.kind, res) {
            let o = 1 - tp;
            let mut out: SVec<SlotRef, 8> = SVec::new();
            let mut blocked: SVec<SlotRef, 8> = SVec::new();
            let mut seen: SVec<SlotRef, 8> = SVec::new();
            for s in sel.iter() {
                if s.p == o && !seen.contains(s) {
                    seen.push(*s);
                    let (e, prevented) = self.run_fx(Effect::TrainerTarget { p: tp, card: tc, target: Some(*s) })?;
                    if prevented || matches!(e, Effect::TrainerTarget { target: None, .. }) {
                        blocked.push(*s);
                    }
                }
            }
            for s in sel.iter() {
                if !blocked.contains(s) {
                    out.push(*s);
                }
            }
            return Ok(Res::Slots(out));
        }
        Ok(res)
    }

    // -----------------------------------------------------------------------
    // Dispatch

    /// `AddPlayerAction` for both players; starts setup.
    pub fn start(&mut self, decks: [&[crate::carddb::DefId]; 2]) -> R {
        for (p, deck) in decks.iter().enumerate() {
            let base = self.st.n_cards;
            for (i, d) in deck.iter().enumerate() {
                let c = base + i as u8;
                self.st.cards[c as usize] = CardInst { def: *d, owner: p as u8, ..Default::default() };
                self.st.players[p].deck.push(c);
            }
            self.st.n_cards += deck.len() as u8;
            for d in deck.iter() {
                if let Some(imp) = cards::impl_for(*d) {
                    self.kinds_present |= imp.mask;
                }
            }
            if !setup::deck_is_valid(&self.st, deck) {
                self.st.players_added = p as u8;
                self.st.phase = GamePhase::Finished;
                self.st.winner = WINNER_NONE;
                return Ok(());
            }
        }
        self.st.phase = GamePhase::Setup;
        setup::start(self)?;
        self.after_dispatch()
    }

    fn after_dispatch(&mut self) -> R {
        self.resolve_wait_items()?;
        if self.items.is_empty() {
            check::check_state(self, OnComplete::None)?;
        }
        self.gc();
        Ok(())
    }

    /// Free effect and temp storage once nothing can reference it.
    fn gc(&mut self) {
        if self.items.is_empty() && self.waits.is_empty() {
            self.fx.clear();
            self.temp_used = [false; 3];
            self.coin_callbacks.clear();
        }
    }

    /// Resolve a prompt (`ResolvePromptAction` with a decoded result).
    pub fn resolve(&mut self, prompt_index: usize, result: Res) -> R {
        let id = self.prompts.as_slice()[prompt_index].id;
        let r = self.reduce_prompt(id, result);
        if r.is_ok() && self.items.is_empty() {
            let r2 = check::check_state(self, OnComplete::None);
            self.gc();
            return r2;
        }
        self.gc();
        r
    }

    /// A turn action on a scratch copy the caller discards on error (no backup).
    pub fn act_trial(&mut self, action: Action) -> R {
        if self.prompts.iter().any(|p| p.result.is_none()) {
            bail!("ACTION_IN_PROGRESS");
        }
        self.items.clear();
        turn::play_card_reducer(self, action)?;
        turn::player_turn_reducer(self, action)?;
        self.after_dispatch()
    }

    /// A turn action (`Store.reduce`): all-or-nothing.
    pub fn act(&mut self, action: Action) -> R {
        if self.prompts.iter().any(|p| p.result.is_none()) {
            bail!("ACTION_IN_PROGRESS");
        }
        let backup = *self;
        self.items.clear();
        let r = (|| {
            turn::play_card_reducer(self, action)?;
            turn::player_turn_reducer(self, action)?;
            self.after_dispatch()
        })();
        if r.is_err() {
            *self = backup;
        }
        r
    }

    // -----------------------------------------------------------------------
    // Driver helpers

    pub fn pending(&self) -> Pending {
        if self.st.phase == GamePhase::Finished {
            return Pending::Finished;
        }
        let mut decision = None;
        for (i, p) in self.prompts.iter().enumerate() {
            if p.result.is_some() {
                continue;
            }
            match p.class() {
                PromptClass::Chance => return Pending::Chance(i),
                PromptClass::Info => return Pending::Info(i),
                PromptClass::Decision => {
                    if decision.is_none() {
                        decision = Some(i);
                    }
                }
            }
        }
        if let Some(i) = decision {
            return Pending::Decision(i);
        }
        if self.st.phase == GamePhase::PlayerTurn {
            return Pending::Turn(self.st.active_player);
        }
        Pending::Stuck
    }

    /// Chance outcome drawn from the internal RNG, as the oracle arbiter does.
    pub fn draw_chance(&mut self, i: usize) -> Res {
        let pr = self.prompts.as_slice()[i];
        match pr.kind {
            PromptKind::ShuffleDeck => {
                let p = self.st.player_index_by_id(pr.perspective_id());
                let n = self.st.players[p].deck.len();
                let mut o = [0u8; 60];
                self.rng.shuffle(n, &mut o);
                Res::Order(List::from_slice(&o[..n]))
            }
            PromptKind::CoinFlip => Res::Bool(self.rng.coin()),
            _ => Res::True,
        }
    }

    /// Resolve chance and info prompts until a decision, turn or the end.
    pub fn settle(&mut self) -> R {
        for _ in 0..10_000 {
            match self.pending() {
                Pending::Chance(i) => {
                    let r = self.draw_chance(i);
                    self.resolve(i, r)?;
                }
                Pending::Info(i) => self.resolve(i, Res::True)?,
                _ => return Ok(()),
            }
        }
        bail!("settle: too many automatic prompts")
    }

    pub fn player_id(&self, p: usize) -> u8 {
        self.st.players[p].id
    }
}

/// `CardList.applyOrder`: ignored unless `order` is a permutation of the list.
pub fn apply_order<L: CardList + ?Sized>(list: &mut L, order: &[u8]) {
    let n = list.len();
    if order.len() != n {
        return;
    }
    let mut seen = [false; 64];
    for &o in order {
        if o as usize >= n || seen[o as usize] {
            return;
        }
        seen[o as usize] = true;
    }
    let copy: SVec<CardId, 60> = {
        let mut v = SVec::new();
        for &c in list.as_slice() {
            v.push(c);
        }
        v
    };
    let s = list.as_mut_slice();
    for (i, &o) in order.iter().enumerate() {
        s[i] = copy.as_slice()[o as usize];
    }
}
