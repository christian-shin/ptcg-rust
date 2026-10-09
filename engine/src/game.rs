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
    /// `order => player.deck.applyOrder(order)` with no follow-up wait.
    ShuffleApplyNoWait { p: u8 },
    Prefab(crate::prefabs::PrefabCont),
    /// Sudden death: first-player coin flip.
    SuddenDeathCoin,
    Card { card: CardId, frame: CardFrame },
    /// Copy-attack generators (`copy_attack.rs`).
    CopyAttack(crate::copy_attack::CopyFrame),
    /// A card continuation created by a source card's code running as
    /// `.call(copycat)` (copy-attack delegation): resumed with the source's port.
    DelegCard { card: CardId, source: CardId, serial: u8, frame: CardFrame },
    /// Little Grudge DiscardEnergyPrompt (KnockOutEffect reducer).
    /// Order of the step 7 triggers: the defending player's choice of the one to resolve first.
    TriggerOrder { atk: EffId },
    LittleGrudge { owner: u8, prize_taker: u8, attack: crate::state::AttackRef, source_card: CardId, target: SlotRef },
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
    Sequence { p: u8, mode: u8, results: u32, n: u8, callback: u8, cause: crate::cause::Cause },
    /// Final callback of a sequence: card receives (bitmask, count) via `frame.a[2..]`.
    SequenceCard { card: CardId, frame: CardFrame },
    /// `Card` / `SequenceCard` created by delegated source code (see `Cont::DelegCard`).
    DelegCard { card: CardId, source: CardId, serial: u8, frame: CardFrame },
    DelegSequenceCard { card: CardId, source: CardId, serial: u8, frame: CardFrame },
    /// `withOptionalCoinFlipCancelTrainer` (Seismitoad 30C's Quaking Fist).
    CancelTrainer { kind: crate::engine::play::TrainerPlayKind, p: u8, card: CardId, target: Option<SlotRef> },
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
    /// Per-object effect fields no core code reads ([`fx_flag`] bits).
    pub flags: u8,
}

/// [`EffSlot::flags`] bits.
pub mod fx_flag {
    /// `DealDamageEffect.damageIncreased` (Hop's Snorlax).
    pub const DAMAGE_INCREASED: u8 = 1 << 0;
    /// `PutDamageEffect.nonstackingDamageReducers` contains 'Curly Wall' (Bouffalant SCR).
    pub const CURLY_WALL: u8 = 1 << 1;
    /// `AttackEffect.afterDamageEffects` is defined (the attack's after-damage window is open).
    pub const AFTER_DMG_OPEN: u8 = 1 << 2;
    /// `AttackEffect.attackTriggers` is defined (the attack's step 7 trigger window is open).
    pub const TRIGGERS_OPEN: u8 = 1 << 3;
}

/// A step queued in an attack's after-damage window (`prefabs/after-damage.ts`).
#[derive(Clone, Copy, Debug)]
pub enum AfterDmgStep {
    /// An effect (held by the queue) to reduce after the damage.
    Fx(EffId),
    /// `MOVE_CARDS(source, destination, { cards, sourceCard })` after the damage (`afterDamageOf`).
    Move { source: ListRef, destination: ListRef, source_card: CardId, cards: SVec<CardId, 64> },
    /// `SHUFFLE_DECK(player)` after the damage.
    Shuffle(u8),
}

/// Step 7 of the attack flow chart: an effect that activates when a Pokémon receives the attack, recorded when the
/// damage is done (`AttackTrigger` in Twinleaf, `prefabs/after-damage.ts`).
#[derive(Clone, Copy, Debug)]
pub struct AtkTrig {
    pub atk: EffId,
    /// The card whose effect triggered.
    pub card: CardId,
    /// The damaged Pokémon.
    pub target: SlotRef,
    pub damage: i32,
    /// The Attacking Pokémon's slot and the Pokémon card that was in it.
    pub source: SlotRef,
    pub source_pokemon: Option<CardId>,
    /// Delayed trap of the damaged Pokémon.
    pub retaliate: Option<crate::state::StoredRetaliate>,
    /// The trigger moves an Energy off the Attacking Pokémon (Handheld Fan).
    pub removes_attacker_energy: bool,
}

/// The attack in progress and what it damaged, kept until the Knock Out check (`prefabs/last-attack.ts`).
#[derive(Clone, Copy, Debug)]
pub struct LastAttack {
    /// The attacking player.
    pub p: u8,
    /// The AttackEffect being reduced (retained until the attack finishes; stale afterwards: read it only while the
    /// attack is in progress, as a Trainer used as the attack's effect does, `spec::ops::board::frame_attack`).
    pub effect: EffId,
    /// The attack (of the AttackEffect being reduced).
    pub attack: AttackRef,
    pub source: SlotRef,
    /// The Pokémon card that used the attack (in `source` when the attack started).
    pub pokemon: Option<CardId>,
    /// Opponent's Pokémon damaged by the attack while in the Active Spot.
    pub damaged_active: SVec<SlotRef, 4>,
    /// Every Pokémon (any zone) that took damage, not counters, from the attack (`RECORD_DAMAGED`).
    pub damaged: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>,
}

#[derive(Clone, Copy, Debug)]
pub struct AfterDmg {
    pub atk: EffId,
    pub step: AfterDmgStep,
}

pub const MAX_FX: usize = 48;
/// Scratch CardLists alive at once (duplicated cards can run one handler several times per effect).
pub const MAX_TEMPS: usize = 32;

/// Player actions (`game-actions.ts`, `play-card-action.ts`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    PlayCard { hand_index: u8, target: CardTarget },
    /// `from`: full name of the Benched Pokémon whose attack this is (Memory Helix), else None.
    Attack { name: &'static str, from: Option<&'static str> },
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

thread_local! {
    static FORK_POOL: std::cell::RefCell<Vec<Box<Game>>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A pooled scratch `Game` from [`Game::fork`]; returned to the pool on drop.
pub struct Fork(Option<Box<Game>>);

impl std::ops::Deref for Fork {
    type Target = Game;
    #[inline]
    fn deref(&self) -> &Game {
        self.0.as_deref().unwrap()
    }
}

impl std::ops::DerefMut for Fork {
    #[inline]
    fn deref_mut(&mut self) -> &mut Game {
        self.0.as_deref_mut().unwrap()
    }
}

impl Drop for Fork {
    fn drop(&mut self) {
        if let Some(b) = self.0.take() {
            let _ = FORK_POOL.try_with(|p| p.borrow_mut().push(b));
        }
    }
}

#[derive(Clone, Copy)]
pub struct Game {
    pub st: State,
    pub rng: Rng,
    pub prompts: SVec<PromptRec, 32>,
    pub last_prompt_id: u32,
    pub items: SVec<PromptItem, 32>,
    pub waits: SVec<Cont, 16>,
    pub fx: SVec<EffSlot, MAX_FX>,
    pub temps: [List<120>; MAX_TEMPS],
    pub temp_used: [bool; MAX_TEMPS],
    /// Callbacks referenced by pending `CoinFlipEffect`s.
    pub coin_callbacks: SVec<CoinCb, 16>,
    /// Trainer whose effect is creating prompts (`Store.resolvingTrainer`).
    pub resolving_trainer: Option<(u8, CardId)>,
    /// `probingStadiumEffect` (stadium-effect.ts module state).
    pub probing_stadium: bool,
    /// A legality trial (options.rs): work that can't change whether the
    /// action is legal is skipped (`PTCG_VERIFY_LEGAL=1` checks this).
    pub trial: bool,
    /// Union of subscription masks of every card in the game (skip propagation otherwise).
    pub kinds_present: crate::effects::KindMask,
    /// The cards of the game that declare an Ability lock (`ActiveLock` / `AbilityLock`), a bit per card id: the
    /// only cards `lock_sync` can stamp (set once by `start`).
    pub lock_cards: [u64; 4],
    /// Opt-in effect-type trace for the diff tool (not part of rules state).
    pub trace_effects: bool,
    /// Copy-attack sessions (`copyAttackSessions`, module state in Twinleaf).
    pub copy_sessions: SVec<crate::copy_attack::CopySession, 8>,
    pub copy_serial: u8,
    /// Source card code currently running as `.call(copycat)`.
    pub deleg: Option<crate::copy_attack::Deleg>,
    /// Energy removals waiting for the damage of an attack (`afterDamageEffects`).
    pub after_dmg: SVec<AfterDmg, 32>,
    /// Step 7 triggers waiting for the end of the attack (`attackTriggers`).
    pub triggers: SVec<AtkTrig, 16>,
    /// Pokémon that survived this attack's damage with "remaining HP becomes 10" (`survive-on-ten.ts`).
    pub ten_hp: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>,
    /// Pokémon (and the owner who flips) whose Tenacious Body / Durable Body coin waits for all the attack's damage
    /// (ruling 1770; `survive-on-ten.ts`).
    pub ten_hp_coin: SVec<(SlotRef, u8), 8>,
    /// `last-attack.ts`: the attack in progress / the last attack, for the Knock Out check.
    pub last_attack: Option<LastAttack>,
    /// Attack choices made at step D, before the damage, by spec cards
    ///: read when the effects are carried out after the damage.
    pub spec_choices: SVec<crate::spec::SpecChoice, 16>,
    /// `lock_sync` is running (it probes Abilities, which must not start another sync).
    pub lock_syncing: bool,
    /// The per-kind dispatch index (`dispatch.rs`).
    pub dispatch: crate::dispatch::DispatchIndex,
    /// The derived layer (`derived.rs`).
    pub derived: crate::derived::Derived,
}

/// How the propagation order ranks the cards: by super type, or specially for these three effects.
pub(crate) const PROP_POWER: u16 = 1;
pub(crate) const PROP_CHECK_POWERS: u16 = 2;
pub(crate) const PROP_AFTER_ATTACK: u16 = 3;

pub(crate) fn prop_class(e: &Effect) -> u16 {
    match e {
        Effect::Power { .. } => PROP_POWER,
        Effect::CheckPokemonPowers { .. } => PROP_CHECK_POWERS,
        Effect::AfterAttack { .. } => PROP_AFTER_ATTACK,
        _ => 0,
    }
}

/// `PTCG_VERIFY_CACHE=1` (or `PTCG_VERIFY_LEGAL=1`): every cached answer (the dispatch index, the quiet
/// CheckHp) is checked against a fresh computation.
pub(crate) fn verify_cache() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| ["PTCG_VERIFY_CACHE", "PTCG_VERIFY_LEGAL"].iter().any(|k| std::env::var(k).map_or(false, |v| v == "1")))
}

/// Prompt constructor work Twinleaf does in the prompt class itself:
/// `ChooseEnergyPrompt` with `allowCancel: false` reduces its cost to what the
/// offered Energy can pay (`getCostThatCanBePaid`; idempotent).
fn construct_prompt(kind: PromptKind) -> PromptKind {
    match kind {
        PromptKind::ChooseEnergy { energy, cost, allow_cancel: false } => {
            let cost = crate::energy::cost_that_can_be_paid(&energy, &cost);
            PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }
        }
        k => k,
    }
}

pub struct EffectLog;

thread_local! {
    pub static EFFECT_TRACE: std::cell::RefCell<Vec<&'static str>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl Game {
    /// A scratch copy of the game for trials: a pooled heap `Game` refilled
    /// with `copy_from` (no allocation or full-size memcpy after warm-up).
    #[inline]
    pub fn fork(&self) -> Fork {
        let b = FORK_POOL.with(|p| p.borrow_mut().pop());
        let b = match b {
            Some(mut b) => {
                b.copy_from(self);
                b
            }
            None => Box::new(*self),
        };
        let mut b = b;
        // A trial's draws are not the game's: they never go into a recorded trace.
        b.rng.set_record(false);
        Fork(Some(b))
    }

    /// Overwrite `self` with `src`, copying only the live part of the
    /// prompt / item / wait / effect stacks. Same value as `*self = *src`,
    /// but most of `size_of::<Game>()` is unused capacity.
    #[inline]
    pub fn copy_from(&mut self, src: &Game) {
        let d: *mut Game = self;
        // SAFETY: `d` is a valid, exclusive Game; SVec tails are MaybeUninit.
        unsafe {
            use std::ptr::addr_of_mut as f;
            let Game {
                st, rng, prompts, last_prompt_id, items, waits, fx, temps, temp_used, coin_callbacks,
                resolving_trainer, probing_stadium, trial, kinds_present, lock_cards, trace_effects, copy_sessions, copy_serial, deleg, after_dmg, triggers, ten_hp, ten_hp_coin, last_attack, spec_choices, lock_syncing,
                dispatch,
                derived: _,
            } = src;
            f!((*d).st).write(*st);
            // The destination keeps its own recording flag (restoring the live game from a backup
            // fork must not turn its recording off).
            let rec = (*d).rng.recording();
            f!((*d).rng).write(*rng);
            (*d).rng.set_record(rec);
            prompts.copy_live_to(f!((*d).prompts));
            f!((*d).last_prompt_id).write(*last_prompt_id);
            items.copy_live_to(f!((*d).items));
            waits.copy_live_to(f!((*d).waits));
            fx.copy_live_to(f!((*d).fx));
            f!((*d).temps).write(*temps);
            f!((*d).temp_used).write(*temp_used);
            coin_callbacks.copy_live_to(f!((*d).coin_callbacks));
            f!((*d).resolving_trainer).write(*resolving_trainer);
            f!((*d).probing_stadium).write(*probing_stadium);
            f!((*d).trial).write(*trial);
            f!((*d).kinds_present).write(*kinds_present);
            f!((*d).lock_cards).write(*lock_cards);
            f!((*d).trace_effects).write(*trace_effects);
            copy_sessions.copy_live_to(f!((*d).copy_sessions));
            f!((*d).copy_serial).write(*copy_serial);
            f!((*d).deleg).write(*deleg);
            after_dmg.copy_live_to(f!((*d).after_dmg));
            triggers.copy_live_to(f!((*d).triggers));
            ten_hp.copy_live_to(f!((*d).ten_hp));
            ten_hp_coin.copy_live_to(f!((*d).ten_hp_coin));
            spec_choices.copy_live_to(f!((*d).spec_choices));
            f!((*d).last_attack).write(*last_attack);
            f!((*d).lock_syncing).write(*lock_syncing);
            // The index comes along with the layout it describes (each entry carries its own generation).
            f!((*d).dispatch).write(*dispatch);
            // The derived facts are not copied: the copy starts stale.
            (*d).derived.invalidate();
        }
    }
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
            temps: [List::new(); MAX_TEMPS],
            temp_used: [false; MAX_TEMPS],
            coin_callbacks: SVec::new(),
            resolving_trainer: None,
            probing_stadium: false,
            trial: false,
            kinds_present: crate::effects::KindMask::EMPTY,
            lock_cards: [0; 4],
            trace_effects: false,
            copy_sessions: SVec::new(),
            copy_serial: 0,
            deleg: None,
            after_dmg: SVec::new(),
            triggers: SVec::new(),
            ten_hp: SVec::new(),
            ten_hp_coin: SVec::new(),
            last_attack: None,
            spec_choices: SVec::new(),
            lock_syncing: false,
            dispatch: crate::dispatch::DispatchIndex::new(),
            derived: crate::derived::Derived::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Effect arena

    pub fn new_fx(&mut self, e: Effect) -> EffId {
        // Events batch 1: the effect's `Cause` against the old inference (VERIFY only).
        crate::cause::verify_effect(self, &e);
        self.fx.push(EffSlot { e, prevent_default: false, refs: 1, flags: 0 });
        (self.fx.len() - 1) as EffId
    }

    pub fn retain_fx(&mut self, id: EffId) {
        self.fx.as_mut_slice()[id as usize].refs += 1;
    }

    pub fn release_fx(&mut self, id: EffId) {
        let s = &mut self.fx.as_mut_slice()[id as usize];
        s.refs = s.refs.saturating_sub(1);
        if s.refs == 0 && s.flags & (fx_flag::AFTER_DMG_OPEN | fx_flag::TRIGGERS_OPEN) != 0 {
            // The attack is gone with its window (a thrown error): drop what waited for the damage.
            self.drop_after_damage(id);
            self.drop_triggers(id);
        }
        while let Some(top) = self.fx.as_slice().last() {
            if top.refs == 0 {
                self.fx.pop();
            } else {
                break;
            }
        }
    }

    /// `OPEN_AFTER_DAMAGE_EFFECTS(attackEffect)`.
    pub fn open_after_damage(&mut self, atk: EffId) {
        self.drop_after_damage(atk);
        self.drop_triggers(atk);
        self.set_fx_flag(atk, fx_flag::AFTER_DMG_OPEN);
        self.set_fx_flag(atk, fx_flag::TRIGGERS_OPEN);
    }

    /// `ATTACKER_OF_KNOCK_OUT`: the Pokémon that used the opponent's attack in progress while a Pokémon of `owner` is
    /// being Knocked Out: the Pokémon card, and the slot it is in now (None when it left play).
    pub fn attacker_of_knock_out(&self, owner: usize) -> Option<(Option<CardId>, Option<SlotRef>)> {
        let la = self.last_attack?;
        if self.st.phase != GamePhase::Attack || self.st.active_player as usize == owner || la.p as usize == owner {
            return None;
        }
        let in_play = la.pokemon.is_some() && self.st.slot_pokemon(la.source.p as usize, la.source.s) == la.pokemon;
        Some((la.pokemon, if in_play { Some(la.source) } else { None }))
    }

    /// `ATTACK_THAT_DAMAGED_KNOCKED_OUT`: as `attacker_of_knock_out`, only for a Knock Out of `target` by damage from
    /// the attack while it was in the Active Spot.
    pub fn attack_that_damaged_knocked_out(&self, owner: usize, target: SlotRef) -> Option<(Option<CardId>, Option<SlotRef>)> {
        let r = self.attacker_of_knock_out(owner)?;
        if !self.last_attack?.damaged_active.contains(&target) {
            return None;
        }
        Some(r)
    }

    /// `KNOCKED_OUT_BY_ATTACK_DAMAGE`: as `attacker_of_knock_out`, only when `target` took damage from the attack, in any
    /// zone (E-04; rulings 648, 674): not for a Knock Out by an effect without damage or by counters.
    pub fn knocked_out_by_attack_damage(&self, owner: usize, target: SlotRef) -> Option<(Option<CardId>, Option<SlotRef>)> {
        let r = self.attacker_of_knock_out(owner)?;
        if !self.last_attack?.damaged.contains(&target) {
            return None;
        }
        Some(r)
    }

    /// The step 7 trigger window of the attack effect is open.
    pub fn triggers_open(&self, atk: EffId) -> bool {
        (atk as usize) < self.fx.len()
            && matches!(self.fx.as_slice()[atk as usize].e, Effect::Attack { .. })
            && self.fx.as_slice()[atk as usize].flags & fx_flag::TRIGGERS_OPEN != 0
    }

    fn drop_triggers(&mut self, atk: EffId) {
        let mut i = 0;
        while i < self.triggers.len() {
            if self.triggers.as_slice()[i].atk == atk {
                self.triggers.remove_at(i);
            } else {
                i += 1;
            }
        }
    }

    /// `ATTACK_TRIGGER(store, state, afterDamageEffect, card, retaliate)`: record the step 7 trigger of `card` on the
    /// damaged Pokémon `b.target` (the attacker is `b.source`); it resolves in `run_attack_triggers`, or at once when
    /// no window is open. Triggers resolve in the order they were recorded.
    pub fn attack_trigger(&mut self, b: AtkBase, damage: i32, card: CardId, retaliate: Option<crate::state::StoredRetaliate>, removes_attacker_energy: bool) -> R {
        let t = AtkTrig {
            atk: b.attack_effect,
            card,
            target: b.target,
            damage,
            source: b.source,
            source_pokemon: self.st.slot_pokemon(b.source.p as usize, b.source.s),
            retaliate,
            removes_attacker_energy,
        };
        if self.triggers_open(b.attack_effect) {
            self.triggers.push(t);
            return Ok(());
        }
        self.resolve_attack_trigger(t)
    }

    fn resolve_attack_trigger(&mut self, t: AtkTrig) -> R {
        let (p, opp, attack) = match *self.e(t.atk) {
            Effect::Attack { p, opp, attack, .. } => (p, opp, attack),
            _ => return Ok(()),
        };
        let source_in_play = t.source_pokemon.is_some() && self.st.slot_pokemon(t.source.p as usize, t.source.s) == t.source_pokemon;
        self.run_fx(Effect::AttackTrigger {
            attack_effect: t.atk,
            p,
            opp,
            attack,
            card: t.card,
            target: t.target,
            damage: t.damage,
            source: t.source,
            source_in_play,
            retaliate: t.retaliate,
        })?;
        Ok(())
    }

    /// There are step 7 triggers of the attack left to resolve.
    pub fn attack_triggers_pending(&self, atk: EffId) -> bool {
        self.triggers.as_slice().iter().any(|t| t.atk == atk)
    }

    fn trigger_indices(&self, atk: EffId) -> SVec<usize, 16> {
        let mut idx: SVec<usize, 16> = SVec::new();
        for (i, t) in self.triggers.as_slice().iter().enumerate() {
            if t.atk == atk {
                idx.push(i);
            }
        }
        idx
    }

    /// Two pending triggers can give different results in a different order: Handheld Fan moves an Energy off the
    /// Attacking Pokémon, which can be the Mist Energy that blocks a delayed trap. All others commute.
    fn trigger_order_matters(&self, idx: &[usize]) -> bool {
        for &i in idx {
            let a = self.triggers.as_slice()[i];
            if !a.removes_attacker_energy {
                continue;
            }
            for &j in idx {
                let b = self.triggers.as_slice()[j];
                if b.retaliate.is_none() || b.source_pokemon.is_none() || self.st.slot_pokemon(b.source.p as usize, b.source.s) != b.source_pokemon {
                    continue;
                }
                if self.st.slot(b.source.p as usize, b.source.s).cards.iter().any(|c| self.st.cdef(c).is_energy() && self.st.cdef(c).name == "Mist Energy") {
                    return true;
                }
            }
        }
        false
    }

    /// `RESOLVE_NEXT_ATTACK_TRIGGER` (Advanced Player's Rulebook E-03): resolve the next recorded step 7 trigger, in the
    /// order recorded, except that the defending player picks the one to resolve first (a Select prompt over the
    /// cards' names) while 2 or more are pending and the order matters. The caller waits for prompts between calls.
    pub fn resolve_next_attack_trigger(&mut self, atk: EffId) -> R {
        let idx = self.trigger_indices(atk);
        if idx.is_empty() {
            return Ok(());
        }
        if idx.len() >= 2 && self.trigger_order_matters(idx.as_slice()) {
            let mut names: SVec<&'static str, 8> = SVec::new();
            for &i in idx.iter() {
                names.push(self.st.cdef(self.triggers.as_slice()[i].card).full_name);
            }
            let opp = match *self.e(atk) {
                Effect::Attack { opp, .. } => opp as usize,
                _ => return Ok(()),
            };
            let pid = self.player_id(opp);
            self.prompt(
                pid,
                "CHOOSE_OPTION",
                crate::prompts::PromptKind::Select { values: crate::prompts::SelectValues::Dyn(names), allow_cancel: false, default_value: 0 },
                Cont::TriggerOrder { atk },
            );
            return Ok(());
        }
        let t = self.triggers.remove_at(idx.as_slice()[0]);
        self.resolve_attack_trigger(t)
    }

    /// The choice of the `TriggerOrder` prompt: resolve that trigger now.
    fn resolve_chosen_trigger(&mut self, atk: EffId, choice: i32) -> R {
        let idx = self.trigger_indices(atk);
        if choice < 0 || choice as usize >= idx.len() {
            crate::bail!("TypeError: trigger is undefined");
        }
        let t = self.triggers.remove_at(idx.as_slice()[choice as usize]);
        self.resolve_attack_trigger(t)
    }

    /// `CLOSE_ATTACK_TRIGGERS`: close the step 7 trigger window of the attack.
    pub fn close_attack_triggers(&mut self, atk: EffId) {
        if (atk as usize) < self.fx.len() {
            self.fx.as_mut_slice()[atk as usize].flags &= !fx_flag::TRIGGERS_OPEN;
        }
        self.drop_triggers(atk);
    }

    /// The after-damage window of the attack effect is open.
    pub fn after_damage_open(&self, atk: EffId) -> bool {
        (atk as usize) < self.fx.len()
            && matches!(self.fx.as_slice()[atk as usize].e, Effect::Attack { .. })
            && self.fx.as_slice()[atk as usize].flags & fx_flag::AFTER_DMG_OPEN != 0
    }

    pub fn push_after_damage(&mut self, atk: EffId, step: AfterDmgStep) {
        self.after_dmg.push(AfterDmg { atk, step });
    }

    /// Drop the steps queued for `atk` without running them.
    fn drop_after_damage(&mut self, atk: EffId) {
        let mut i = 0;
        while i < self.after_dmg.len() {
            if self.after_dmg.as_slice()[i].atk == atk {
                let d = self.after_dmg.remove_at(i);
                if let AfterDmgStep::Fx(id) = d.step {
                    self.release_fx(id);
                }
            } else {
                i += 1;
            }
        }
    }

    /// `RUN_AFTER_DAMAGE_EFFECTS(state, attackEffect)`: close the window, then run the queued steps in order.
    pub fn run_after_damage(&mut self, atk: EffId) -> R {
        if (atk as usize) < self.fx.len() {
            self.fx.as_mut_slice()[atk as usize].flags &= !fx_flag::AFTER_DMG_OPEN;
        }
        let mut steps: SVec<AfterDmgStep, 32> = SVec::new();
        let mut i = 0;
        while i < self.after_dmg.len() {
            if self.after_dmg.as_slice()[i].atk == atk {
                steps.push(self.after_dmg.remove_at(i).step);
            } else {
                i += 1;
            }
        }
        for step in steps.iter() {
            match *step {
                AfterDmgStep::Move { source, destination, source_card, cards } => {
                    self.run_fx(Effect::MoveCards {
                        source,
                        destination,
                        cards: Some(List::from_slice(cards.as_slice())),
                        count: None,
                        to_top: false,
                        to_bottom: false,
                        skip_cleanup: false,
                        source_card,
                    })?;
                }
                AfterDmgStep::Fx(id) => {
                    let r = self.reduce_effect(id);
                    self.release_fx(id);
                    r?;
                }
                AfterDmgStep::Shuffle(p) => crate::prefabs::shuffle_deck(self, p as usize),
            }
        }
        Ok(())
    }

    /// `DEFER_UNTIL_AFTER_DAMAGE(store, effect)`: queue an Energy removal of an attack whose window is open.
    fn defer_after_damage(&mut self, id: EffId) -> bool {
        let atk = match *self.e(id) {
            Effect::MoveOpponentEnergy { b, card, .. } => {
                if !self.st.cdef(card).is_energy() {
                    return false;
                }
                b.attack_effect
            }
            Effect::DiscardCards { b, ref cards } | Effect::CardsToHand { b, ref cards } => {
                if cards.is_empty() || !cards.iter().all(|c| self.st.cdef(*c).is_energy()) {
                    return false;
                }
                b.attack_effect
            }
            _ => return false,
        };
        if !self.after_damage_open(atk) {
            return false;
        }
        self.retain_fx(id);
        self.push_after_damage(atk, AfterDmgStep::Fx(id));
        true
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
    pub fn fx_flags(&self, id: EffId) -> u8 {
        self.fx.as_slice()[id as usize].flags
    }
    pub fn set_fx_flag(&mut self, id: EffId, bit: u8) {
        self.fx.as_mut_slice()[id as usize].flags |= bit;
    }
    pub fn set_prevent(&mut self, id: EffId, v: bool) {
        self.fx.as_mut_slice()[id as usize].prevent_default = v;
    }

    /// A coin flip of player `p` for a card's effect (`cause`), whose result goes to `cb`: the B4-OLD request
    /// (`CoinFlipRequest`, which Backtrack Badge's re-flip hook replaces); the flip is a CoinFlip event.
    pub fn coin_flip(&mut self, p: usize, cb: CoinCb, cause: crate::cause::Cause) -> R<Option<bool>> {
        let cb = self.tag_coin(cb);
        self.coin_callbacks.push(cb);
        let k = (self.coin_callbacks.len() - 1) as u8;
        let (e, _) = self.run_fx(Effect::CoinFlipRequest { p: p as u8, callback: Some(k), result: None, skip_reflip_stadium: false, skip_reflip_tool: false, cause })?;
        Ok(match e {
            Effect::CoinFlipRequest { result, .. } => result,
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

    /// [`Game::run_fx`] for a caller that does not read the result (no copy of the effect).
    pub fn run_fx_unit(&mut self, e: Effect) -> R {
        let id = self.new_fx(e);
        let r = self.reduce_effect(id);
        self.release_fx(id);
        r
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
        let kind = construct_prompt(kind);
        let cont = self.tag_cont(cont);
        let id = self.next_id();
        // Safety cap: a card duplicating itself in a loop would get here (the Dangle Tail aliasing that
        // used to is fixed, W1-E; there are no approved divergences for it any more).
        assert!(self.prompts.len() < self.prompts.capacity() && self.items.len() < self.items.capacity(), "prompt stack exhausted");
        crate::expect::on_prompt(self, &kind, matches!(cont, Cont::CoinFlipWait { .. }));
        self.prompts.push(PromptRec { id, player_id, perspective: None, message, kind, result: None, trainer: self.resolving_trainer });
        let mut ids = SVec::new();
        ids.push(id);
        self.items.push(PromptItem { ids, cont });
    }

    /// `store.prompt(state, [prompts...], then)`.
    pub fn prompt_group(&mut self, prompts: &[(u8, &'static str, PromptKind)], cont: Cont) {
        let cont = self.tag_cont(cont);
        let mut ids = SVec::new();
        for &(player_id, message, kind) in prompts {
            let id = self.next_id();
            let kind = construct_prompt(kind);
            self.prompts.push(PromptRec { id, player_id, perspective: None, message, kind, result: None, trainer: self.resolving_trainer });
            ids.push(id);
        }
        self.items.push(PromptItem { ids, cont });
    }

    pub fn wait_prompt(&mut self, cont: Cont) {
        let cont = self.tag_cont(cont);
        self.waits.push(cont);
    }

    /// Continuations created while delegated source code runs belong to the
    /// source's port (Twinleaf closures capture the source's code).
    pub fn tag_cont(&self, c: Cont) -> Cont {
        match (self.deleg, c) {
            (Some(d), Cont::Card { card, frame }) if card == d.copycat => Cont::DelegCard { card, source: d.source, serial: d.serial, frame },
            _ => c,
        }
    }

    pub fn tag_coin(&self, cb: CoinCb) -> CoinCb {
        match (self.deleg, cb) {
            (Some(d), CoinCb::Card { card, frame }) if card == d.copycat => CoinCb::DelegCard { card, source: d.source, serial: d.serial, frame },
            (Some(d), CoinCb::SequenceCard { card, frame }) if card == d.copycat => {
                CoinCb::DelegSequenceCard { card, source: d.source, serial: d.serial, frame }
            }
            _ => cb,
        }
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
            Cont::TriggerOrder { atk } => self.resolve_chosen_trigger(atk, first.as_int()),
            Cont::LittleGrudge { owner, prize_taker, attack, source_card, target } => crate::engine::game_effect::little_grudge_cont(self, owner, prize_taker, attack, source_card, target, first),
            Cont::ChooseActive { p } => check::choose_active_cont(self, p, first),
            Cont::BenchShrink { p, empty } => check::bench_shrink_cont(self, p, empty, first),
            Cont::BetweenTurnsWait { oc } => phase::run_between_turns_effects(self, oc),
            Cont::BetweenTurnsCheck { oc } => check::check_state(self, oc),
            Cont::BurnFlip { p, .. } => {
                let p = p as usize;
                crate::engine::condition::coin_flipped(self, p, crate::spec::event::CoinPurpose::Burned, first.as_bool(), crate::engine::condition::by_condition(p))?;
                if first.as_bool() {
                    phase::checkup_recovers(self, p, SpecialCondition::Burned)?;
                }
                Ok(())
            }
            Cont::SleepFlips { p, .. } => {
                let p = p as usize;
                for r in results.iter() {
                    crate::engine::condition::coin_flipped(self, p, crate::spec::event::CoinPurpose::Asleep, r.as_bool(), crate::engine::condition::by_condition(p))?;
                }
                if results.iter().all(|r| r.as_bool()) {
                    phase::checkup_recovers(self, p, SpecialCondition::Asleep)?;
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
            Cont::ShuffleApplyNoWait { p } => {
                if let Res::Order(o) = first {
                    apply_order(&mut self.st.players[p as usize].deck, o.as_slice());
                }
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
            Cont::SuddenDeathCoin => {
                // The flip for who goes first in the Sudden Death game (player 1 flips; heads, they go first).
                crate::engine::condition::coin_flipped(self, 0, crate::spec::event::CoinPurpose::FirstPlayer, first.as_bool(), crate::cause::Cause::rule(crate::cause::RuleWhich::Setup, 0))?;
                check::setup_sudden_death_game(self, if first.as_bool() { 0 } else { 1 })
            }
            Cont::Card { card, frame } => cards::resume(self, card, frame, results),
            Cont::CopyAttack(f) => crate::copy_attack::resume(self, f, first),
            Cont::DelegCard { card, source, serial, frame } => crate::copy_attack::resume_deleg(self, card, source, serial, frame, results, None),
        }
    }

    pub fn run_coin_cb(&mut self, cb: CoinCb, result: bool) -> R {
        match cb {
            CoinCb::None => Ok(()),
            CoinCb::Card { card, frame } => cards::coin_result(self, card, frame, result),
            CoinCb::DelegCard { card, source, serial, frame } => crate::copy_attack::resume_deleg(self, card, source, serial, frame, &[], Some(result)),
            CoinCb::DelegSequenceCard { card, source, serial, frame } => crate::copy_attack::resume_deleg(self, card, source, serial, frame, &[], None),
            CoinCb::Attack(a) => attack::coin_cb(self, a, result),
            CoinCb::CancelTrainer { kind, p, card, target } => crate::engine::play::cancel_trainer_coin(self, kind, p, card, target, result),
            CoinCb::Sequence { p, mode, results, n, callback, cause } => {
                let results = if result { results | (1 << n) } else { results };
                let n = n + 1;
                let more = if mode == 0 { result } else { n < mode };
                if more {
                    let cb = CoinCb::Sequence { p, mode, results, n, callback, cause };
                    // The sequence reuses its own slot (the flip copies its callback when it resolves, and
                    // sequences skip the reflip offers that would read it again), so "flip until tails"
                    // takes one slot however many heads come up.
                    let own = self.coin_callbacks.as_slice().iter().rposition(|c| matches!(c, CoinCb::Sequence { callback: c2, .. } if *c2 == callback));
                    let k = match own {
                        Some(i) => {
                            self.coin_callbacks.as_mut_slice()[i] = cb;
                            i as u8
                        }
                        None => {
                            self.coin_callbacks.push(cb);
                            (self.coin_callbacks.len() - 1) as u8
                        }
                    };
                    self.run_fx_unit(Effect::CoinFlipRequest { p, callback: Some(k), result: None, skip_reflip_stadium: true, skip_reflip_tool: true, cause })?;
                    return Ok(());
                }
                // Reflip offers (stadium/tool) are applied by those cards' handlers
                // (a card that wraps the final callback, e.g. Backtrack Badge).
                self.finish_coin_sequence(callback, results, n, result)
            }
            CoinCb::SequenceCard { card, frame } => cards::resume(self, card, frame, &[]),
        }
    }

    /// The final step of a coin flip sequence: runs coin callback `callback`
    /// with the results (bit i = flip i heads), the flip count and the last flip.
    pub fn finish_coin_sequence(&mut self, callback: u8, results: u32, n: u8, result: bool) -> R {
        let fin = self.coin_callbacks.as_slice()[callback as usize];
        match fin {
            // The frame stays as the card left it; the results come as `[bits, count]`.
            CoinCb::SequenceCard { card, frame } => cards::resume(self, card, frame, &[Res::Int(results as i32), Res::Int(n as i32)]),
            CoinCb::DelegSequenceCard { card, source, serial, frame } => {
                crate::copy_attack::resume_deleg(self, card, source, serial, frame, &[Res::Int(results as i32), Res::Int(n as i32)], None)
            }
            other => self.run_coin_cb(other, result),
        }
    }

    // -----------------------------------------------------------------------
    // Effect propagation

    // The broadcast: the propagation order computed from the whole board. The dispatch index
    // (`dispatch.rs`) keeps these lists; VERIFY checks every index answer against this.

    /// Cards with a handler for this effect kind, in Twinleaf's `propagateEffect` order (zone order,
    /// then stable sort by rank), the slow way: every card of every zone looked up.
    pub(crate) fn propagation_order_slow(&self, class: u16, kind: u32) -> SVec<CardId, 120> {
        let mut set = 0u128;
        for c in 0..self.st.n_cards {
            if cards::impl_for(self.st.cards[c as usize].def).is_some_and(|imp| imp.mask.has(kind)) {
                set |= 1u128 << c;
            }
        }
        self.propagation_order_fresh(class, set)
    }

    /// The propagation order of the cards in `handlers` (a bit per card id).
    pub(crate) fn propagation_order_fresh(&self, class: u16, handlers: u128) -> SVec<CardId, 120> {
        let mut cards: SVec<CardId, 120> = SVec::new();
        let add = |c: CardId, cards: &mut SVec<CardId, 120>| {
            if (handlers >> c) & 1 != 0 {
                cards.push(c);
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
            match class {
                PROP_POWER => match d.super_type {
                    2 => 0,
                    3 => 1,
                    _ => 2,
                },
                PROP_CHECK_POWERS => match d.super_type {
                    1 => 0,
                    3 => 1,
                    2 if d.trainer_type == TrainerType::Stadium as u8 => 3,
                    _ => 2,
                },
                // AfterAttackEffect: Pokémon, then Energy, then Trainers (R7F-10).
                PROP_AFTER_ATTACK => match d.super_type {
                    1 => 0,
                    3 => 1,
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
        // Energy removed as an effect of an attack waits for the damage (prefabs/after-damage.ts).
        if self.defer_after_damage(id) {
            return Ok(());
        }
        if let Effect::Attack { p, source, attack, .. } = *self.e(id) {
            self.ten_hp = SVec::new();
            self.ten_hp_coin = SVec::new();
            self.last_attack = Some(LastAttack {
                p,
                effect: id,
                source,
                attack,
                pokemon: self.st.slot_pokemon(source.p as usize, source.s),
                damaged_active: SVec::new(),
                damaged: SVec::new(),
            });
        }
        if self.trace_effects {
            let t = self.e(id).type_name();
            EFFECT_TRACE.with(|v| v.borrow_mut().push(t));
        }
        // A copied attack gives the copycat the attack only: source code run for
        // the copycat can't use or probe an Ability of the copycat (store.ts).
        if let Effect::Power { card, .. } = *self.e(id) {
            if matches!(self.deleg, Some(d) if d.attacks && d.copycat == card) {
                return Err(GameError("BLOCKED_BY_EFFECT"));
            }
        }

        // Propagate to cards.
        let kind = self.e(id).kind();
        // B6-OLD: an attack-effect probe a lasting prevention on its target answers ("during your opponent's next turn,
        // prevent all effects of attacks done to this Pokémon"; `passive::B6OLD_PROBES`); the in-play preventions answer it
        // on its dispatch.
        if crate::spec::passive::B6OLD_PROBE_MASK.has(kind) && crate::spec::passive::probe_lasting_prevented(self, id)? {
            self.set_prevent(id, true);
        }
        let class = prop_class(self.e(id));
        // The batch 4 / 5 events have no dispatch handler: their declarations are read through the index.
        let order = if self.kinds_present.has(kind) && !crate::spec::event::INDEX_ONLY_EVENT_KINDS.has(kind) { self.listeners(class, kind) } else { SVec::new() };
        for &c in order.iter() {
            self.call_card(c, id, kind)?;
        }
        if !self.copy_sessions.is_empty() {
            crate::copy_attack::resolve_sessions(self, id)?;
        }

        if self.prevented(id) {
            return Ok(());
        }

        phase::reducer(self, id)?;
        if matches!(kind, k::GAIN_CONDITION | k::REMOVE_CONDITION | k::HEAL | k::COIN_FLIP) {
            crate::engine::condition::reducer(self, id)?;
        }
        if matches!(kind, k::ATTACH | k::MOVE_ENERGY | k::MOVE_TOOL) {
            crate::engine::attach::reducer(self, id)?;
        }
        if kind == k::CHANGE_ACTIVE {
            crate::engine::change_active::reducer(self, id)?;
        }
        if kind == k::PLACE_COUNTERS {
            crate::engine::damage::reducer(self, id)?;
        }
        if matches!(kind, k::ENTER_PLAY | k::EVOLVE) {
            crate::engine::enter::reducer(self, id)?;
        }
        play::play_trainer_reducer(self, id)?;
        retreat::reducer(self, id)?;
        crate::engine::game_effect::reducer(self, id)?;
        attack::reducer(self, id)?;
        check::check_state_reducer(self, id)?;
        if crate::derived::INVALIDATING_KINDS.has(kind) {
            self.derived.invalidate();
        }
        if matches!(
            kind,
            k::MOVE_CARDS
                | k::ENTER_PLAY
                | k::EVOLVE
                | k::DEVOLVE
                | k::PLAY_STADIUM
                | k::ATTACH_POKEMON_TOOL
                | k::CHANGE_ACTIVE
                | k::CHECK_TABLE_STATE
        ) && !(kind == k::ENTER_PLAY && self.st.phase == GamePhase::Setup)
        {
            // The Pokémon put down at setup take hold together when setup ends (`setup::finish`).
            crate::spec::passive::lock_sync(self);
        } else if matches!(kind, k::ATTACH | k::MOVE_ENERGY | k::MOVE_TOOL) {
            // Only when an attached card can turn a lock source's Ability on or off.
            crate::spec::passive::lock_sync_attached(self);
        }
        if crate::spec::event::EVENT_KINDS.has(kind) {
            crate::spec::run::after_event(self, id)?;
        }
        Ok(())
    }

    #[inline]
    fn call_card(&mut self, c: CardId, id: EffId, kind: u32) -> R {
        if self.deleg.is_some() {
            // A card's own handler never runs as a delegate.
            let saved = self.deleg.take();
            let r = self.call_card(c, id, kind);
            self.deleg = saved;
            return r;
        }
        let d = self.st.cards[c as usize].def;
        if let Some(imp) = cards::impl_for(d) {
            if imp.mask.has(kind) {
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
            let mut out: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }> = SVec::new();
            let mut blocked: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }> = SVec::new();
            let mut seen: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }> = SVec::new();
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
                if cards::spec_for(*d).map_or(false, |s| crate::spec::passive::declares_ability_lock(s.passives)) {
                    self.lock_cards[(c >> 6) as usize] |= 1u64 << (c & 63);
                }
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
            self.temp_used = [false; MAX_TEMPS];
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
        self.act_with(action, true)
    }

    /// [`Game::act`] for a driver that gives up the game on an error (self-play, the benchmark): no backup
    /// copy, so a rejected action leaves the game half done.
    pub fn act_no_rollback(&mut self, action: Action) -> R {
        self.act_with(action, false)
    }

    fn act_with(&mut self, action: Action, rollback: bool) -> R {
        if self.prompts.iter().any(|p| p.result.is_none()) {
            bail!("ACTION_IN_PROGRESS");
        }
        if !rollback {
            self.items.clear();
            turn::play_card_reducer(self, action)?;
            turn::player_turn_reducer(self, action)?;
            return self.after_dispatch();
        }
        let backup = self.fork();
        self.items.clear();
        let r = (|| {
            turn::play_card_reducer(self, action)?;
            turn::player_turn_reducer(self, action)?;
            self.after_dispatch()
        })();
        if r.is_err() {
            self.copy_from(&backup);
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
                let mut o = [0u8; 120];
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
    let mut seen = [false; 128];
    for &o in order {
        if o as usize >= n || seen[o as usize] {
            return;
        }
        seen[o as usize] = true;
    }
    let copy: SVec<CardId, 120> = {
        let mut v = SVec::new();
        for &c in list.as_slice() {
            v.push(c);
        }
        v
    };
    let s = list.as_permutable_slice();
    for (i, &o) in order.iter().enumerate() {
        s[i] = copy.as_slice()[o as usize];
    }
}
