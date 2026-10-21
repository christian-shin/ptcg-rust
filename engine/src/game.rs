//! The store: effect propagation, prompts with continuations, dispatch.
//!
//! Continuations are data ([`Cont`]) held in the state, so a game with pending prompts can be cloned and resumed. A
//! routine that must wait for an answer is an explicit frame that is moved into the continuation of the prompt it
//! asks.

use crate::cards::{self, CardFrame};
use crate::effects::*;
use crate::engine::{attack, phase, retreat, setup, turn};
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
    CheckState(crate::engine::knockout::CheckFrame),
    TakePrizes { p: u8, cause: [u8; 4] },
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
    /// A re-flip offer's answer (`engine::condition::reflip_answer`).
    Reflip(crate::engine::condition::ReflipAsk),
    TrainerCleanup { p: u8, card: CardId },
    ShuffleApply { p: u8 },
    /// Applies the shuffled order to the deck, with no follow-up wait.
    ShuffleApplyNoWait { p: u8 },
    Prefab(crate::prefabs::PrefabCont),
    /// Sudden death: first-player coin flip.
    SuddenDeathCoin,
    Card { card: CardId, frame: CardFrame },
    /// Copy-attack frames (`copy_attack.rs`).
    CopyAttack(crate::copy_attack::CopyFrame),
    /// A card continuation created by a source card's code running as
    /// `.call(copycat)` (copy-attack delegation): resumed with the source's port.
    DelegCard { card: CardId, source: CardId, serial: u8, frame: CardFrame },
    /// The order of several simultaneous triggers (APR E-03, E-04, E-21, F step 2): the window's chooser picks which
    /// pending effect applies next (`engine::order`).
    OrderEffects { w: crate::engine::order::Window },
    /// The ordered effect's prompts are answered: the next step of the window (`engine::order::next`).
    OrderNext { w: crate::engine::order::Window },
    /// The Attacking Pokémon's Energy to discard when a lasting "when this Pokémon is Knocked Out" trigger applies
    /// (`engine::game_effect::discard_attacker_energy_cont`): the Knocked Out Pokémon's owner `owner`, the effect's source
    /// `source_card` and its attack.
    DiscardAttackerEnergy { owner: u8, attack: crate::state::AttackRef, source_card: CardId, target: SlotRef },
    /// The owner chose which survive-on-10 replacement of a group applies first (`damage::survive_order_chosen`).
    SurviveOrder { grp: crate::engine::damage::SurviveGroup },
    /// A what-if attack is over (`attack::finish_attack`): its Knock Out check (attack step 7, APR D), with no end of the
    /// turn.
    WhatIfOver,
}

/// What happens when a state check completes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnComplete {
    None,
    /// EndTurn: expire KO-time effects, then the next turn starts.
    AfterEndTurn { p: u8 },
    /// The time between turns is over: the next turn is initialized unless the game is finished.
    InitNextTurn,
    /// The end of player `p`'s attack (attack step 7, Knock Outs checked; `attack::attack_over`): the copied attack ends,
    /// then the end of the turn unless the game is over.
    AfterAttack { p: u8 },
}

/// What happens when a coin flip is resolved.
#[derive(Clone, Copy, Debug)]
pub enum CoinCb {
    None,
    Card { card: CardId, frame: CardFrame },
    Attack(attack::AttackCoinCb),
    /// RUN_COIN_FLIP_SEQUENCE: one flip done; `results` bit i = flip i heads. `offer`: a re-flip is offered when the
    /// sequence ends (`engine::condition::flip_sequence`).
    Sequence { p: u8, mode: u8, results: u32, n: u8, callback: u8, cause: crate::cause::Cause, offer: bool },
    /// One flip whose re-flip is offered after its wait; then the request's callback `callback` (`engine::condition::flip_coin`).
    Reflip { p: u8, callback: u8, cause: crate::cause::Cause, offer: crate::engine::condition::ReflipOffer },
    /// Final callback of a sequence: card receives (bitmask, count) via `frame.a[2..]`.
    SequenceCard { card: CardId, frame: CardFrame },
    /// `Card` / `SequenceCard` created by delegated source code (see `Cont::DelegCard`).
    DelegCard { card: CardId, source: CardId, serial: u8, frame: CardFrame },
    DelegSequenceCard { card: CardId, source: CardId, serial: u8, frame: CardFrame },
    /// The coin of a coin-gated lock on a Trainer play (Seismitoad 30C's Quaking Fist; `engine::play_trainer::gate_coin`).
    PlayGate { p: u8, card: CardId, target: Option<SlotRef> },
}

#[derive(Clone, Copy, Debug)]
pub struct PromptItem {
    pub ids: SVec<u32, 4>,
    pub cont: Cont,
    /// A question with no answer it can take, not asked (`Game::prompt`): it waits as an information prompt in its place
    /// and its continuation gets no answer (nothing chosen), so the op that asked does nothing.
    pub unasked: bool,
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
    /// A non-stacking damage bonus was applied to the attack's damage (Hop's Snorlax; also `passive::DamageAcc::flags`).
    pub const DAMAGE_INCREASED: u8 = 1 << 0;
    /// The attack's after-damage window is open.
    pub const AFTER_DMG_OPEN: u8 = 1 << 2;
    /// The attack's step 7 trigger window is open.
    pub const TRIGGERS_OPEN: u8 = 1 << 3;
    /// The player took the attack's optional effect (`AttackFlagKind::OptionChosen`).
    pub const OPTION_CHOSEN: u8 = 1 << 4;
}

/// A step queued in an attack's after-damage window.
#[derive(Clone, Copy, Debug)]
pub enum AfterDmgStep {
    /// An attack's MoveEnergy of `card` from the Pokémon in `from` to the one in `to`, by `cause`, after the damage
    /// (`engine::attach::move_attached`; checked when it runs).
    MoveAttached { card: CardId, from: SlotRef, to: SlotRef, cause: crate::cause::Cause },
    /// The LeavePlay of cards attached to the Pokémon in `target` for `zone` of their owner, by `cause`, after the damage
    /// (`engine::knockout::leave_play_cards`; checked when it runs).
    LeaveCards { target: SlotRef, cards: SVec<CardId, 64>, zone: crate::spec::event::RulesZone, cause: crate::cause::Cause },
    /// `SHUFFLE_DECK(player)` after the damage.
    Shuffle(u8),
}

/// The attack in progress and what it damaged, kept until the Knock Out check.
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

/// Player actions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    PlayCard { hand_index: u8, target: CardTarget },
    /// The attack `attack` of the Active Pokémon's list (its own printed attacks, then the derived ones). `copied`: the
    /// attack is a Benched Pokémon's, used as the Active Pokémon's own (`attack.card` is that Benched Pokémon).
    Attack { attack: AttackRef, copied: bool },
    /// The Ability `power` of the Pokémon at `target` (`power.card` is the card at that spot).
    UseAbility { power: PowerRef, target: CardTarget },
    UseStadium,
    /// Retreat the Active Pokémon (APR A-03): the cost is paid, then the Benched Pokémon to send to the Active Spot is
    /// chosen (a prompt), so the action names none.
    Retreat,
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
    /// One past the highest staging list ever allocated in this line of play: `copy_from` copies only those (the others
    /// were never written, so nothing reads them).
    pub temp_hw: u8,
    /// Callbacks referenced by pending coin flips.
    pub coin_callbacks: SVec<CoinCb, 16>,
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
    /// The copy-attack sessions open now (an attack that uses another Pokémon's attack as its own).
    pub copy_sessions: SVec<crate::copy_attack::CopySession, 8>,
    pub copy_serial: u8,
    /// The attacks being used in the attack in progress by copying: the attack of each copy attack that made its choice
    /// (the copying attack, then each copied attack that copies in turn). None of them can be chosen again in that attack
    /// (`copy_attack::blocked_attacks`, compendium id657); cleared when the attack is over.
    pub copy_chain: SVec<AttackRef, { crate::copy_attack::MAX_COPY_CHAIN }>,
    /// Source card code currently running as `.call(copycat)`.
    pub deleg: Option<crate::copy_attack::Deleg>,
    /// Energy removals waiting for the damage of an attack.
    pub after_dmg: SVec<AfterDmg, 32>,
    /// The effects of the simultaneous-trigger windows waiting to be applied (`engine::order`: the step 7 triggers of the
    /// attack, the Knock Out step 2 effects, the end-of-turn and Checkup effects).
    pub pending: SVec<crate::engine::order::Pending, 32>,
    /// Pokémon that survived this attack's damage with "remaining HP becomes 10".
    pub ten_hp: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>,
    /// Pokémon (and the owner who flips) whose Tenacious Body / Durable Body coin waits for all the attack's damage
    /// (ruling 1770).
    pub ten_hp_coin: SVec<(SlotRef, u8), 8>,
    /// The attack in progress / the last attack, for the Knock Out check.
    pub last_attack: Option<LastAttack>,
    /// Attack choices made at step D, before the damage, by spec cards
    ///: read when the effects are carried out after the damage.
    pub spec_choices: SVec<crate::spec::SpecChoice, 16>,
    /// The derived layer (`derived.rs`).
    pub derived: crate::derived::Derived,
    /// The declaration index (`decl.rs`): per reader family, the cards of the game that declare one (set by `start`).
    pub decls: crate::decl::DeclIndex,
    /// The readers' card lists of the current layout generation (`decl::OrderCache`; not copied by `copy_from`).
    pub order_cache: crate::decl::OrderCache,
    /// The large payloads of the effects in flight (`effects::EffectPayloads`).
    pub fxp: crate::effects::EffectPayloads,
    /// The answers of the prompts answered while others of their group wait (by prompt id; batch 8 C8: out of
    /// `PromptRec`).
    pub answers: SVec<(u32, Res), 8>,
    /// The prompts' large payloads (`prompts::PromptPayload`, by serial; batch 8 C8c), while a prompt refers to them.
    pub prompt_payloads: SVec<(u16, crate::prompts::PromptPayload), 16>,
    pub payload_serial: u16,
    /// The end-of-turn order prompt is open: the end of the turn continues after it (`phase::end_turn_event`).
    pub order_pending: bool,
    /// Pokémon Checkup holds its effects back to order them (two or more apply; `phase`).
    pub checkup_held: bool,
    /// The card whose program is putting Pokémon into play (`engine::enter::put_into_play`): the triggers over their EnterPlay
    /// events wait in its put-into-play window until the program is done (`engine::order::after_effect`).
    pub put_by: Option<CardId>,
    /// Events a prevention stopped so far (`derived::event_prevented`; read by `PTCG_VERIFY_LEGAL=1` only).
    pub prevented_events: u32,
    /// `PTCG_VERIFY_LEGAL=1` only: the cards whose Ability-origin declarations `passive::locate` treats as not in place,
    /// in the copy the continuous-effect assertion recomputes a fact on (`derived::verify_reach`); 0 otherwise.
    pub reach_skip: u128,
    /// The survive-on-10 groups of the attack in progress waiting for the owner's order (`damage::resolve_survive_groups`).
    pub survive_groups: SVec<crate::engine::damage::SurviveGroup, 2>,
    /// The game record (`record.rs`): off unless `record_on` is called; not part of the rules state.
    pub record: crate::record::Recorder,
}

/// How the propagation order ranks the cards: by super type, or specially for the use of a power, the powers a Pokémon has
/// now (`passive::powers_now`, the order of the Ability locks; it was the CheckPokemonPowers effect's) and AfterAttack.
pub(crate) const PROP_POWER: u16 = 1;
pub(crate) const PROP_POWERS_NOW: u16 = 2;
pub(crate) const PROP_AFTER_ATTACK: u16 = 3;

/// The rank of card `c` in the propagation order of rank class `class` (a stable sort by it follows the zone order).
pub(crate) fn prop_rank(g: &Game, class: u16, c: CardId) -> u8 {
    prop_rank_st(&g.st, class, c)
}

pub(crate) fn prop_rank_st(st: &State, class: u16, c: CardId) -> u8 {
    let d = st.cdef(c);
    match class {
        PROP_POWER => match d.super_type {
            2 => 0,
            3 => 1,
            _ => 2,
        },
        PROP_POWERS_NOW => match d.super_type {
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
}

pub(crate) fn prop_class(e: &Effect) -> u16 {
    match e {
        Effect::Power { .. } => PROP_POWER,
        Effect::AfterAttack { .. } => PROP_AFTER_ATTACK,
        _ => 0,
    }
}

/// `PTCG_VERIFY_CACHE=1` (or `PTCG_VERIFY_LEGAL=1`): every cached answer (the derived facts, the state check's fast
/// paths) is checked against a fresh computation.
pub(crate) fn verify_cache() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| ["PTCG_VERIFY_CACHE", "PTCG_VERIFY_LEGAL"].iter().any(|k| std::env::var(k).map_or(false, |v| v == "1")))
}

/// Work a prompt does when it is built: a choose-Energy prompt that cannot be cancelled reduces its cost to what the
/// offered Energy can pay (idempotent).
fn construct_prompt(g: &Game, kind: PromptKind) -> PromptKind {
    match kind {
        PromptKind::ChooseEnergy { energy, cost, allow_cancel: false } => {
            let cost = crate::energy::cost_that_can_be_paid(g.payload_energy(energy), &cost);
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
        // A trial's draws are not the game's: they never go into a recorded trace. Nor its events into the game record.
        b.rng.set_record(false);
        b.record_stop_in_copy();
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
                st, rng, prompts, last_prompt_id, items, waits, fx, temps, temp_used, temp_hw, coin_callbacks,
                trial, kinds_present, lock_cards, trace_effects, copy_sessions, copy_serial, copy_chain, deleg, after_dmg, pending, ten_hp, ten_hp_coin, last_attack, spec_choices,
                derived: _,
                decls,
                order_cache: _,
                fxp,
                answers,
                prompt_payloads,
                payload_serial,
                prevented_events,
                reach_skip,
                survive_groups,
                order_pending,
                checkup_held,
                put_by,
                record,
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
            // The staging lists ever allocated (batch 8 C8: not the 32 x 121 B of them).
            let hw = *temp_hw as usize;
            std::ptr::copy_nonoverlapping(temps.as_ptr(), f!((*d).temps) as *mut List<120>, hw);
            f!((*d).temp_used).write(*temp_used);
            f!((*d).temp_hw).write(*temp_hw);
            coin_callbacks.copy_live_to(f!((*d).coin_callbacks));
            f!((*d).trial).write(*trial);
            f!((*d).kinds_present).write(*kinds_present);
            f!((*d).lock_cards).write(*lock_cards);
            f!((*d).trace_effects).write(*trace_effects);
            copy_sessions.copy_live_to(f!((*d).copy_sessions));
            f!((*d).copy_serial).write(*copy_serial);
            copy_chain.copy_live_to(f!((*d).copy_chain));
            f!((*d).deleg).write(*deleg);
            after_dmg.copy_live_to(f!((*d).after_dmg));
            pending.copy_live_to(f!((*d).pending));
            ten_hp.copy_live_to(f!((*d).ten_hp));
            ten_hp_coin.copy_live_to(f!((*d).ten_hp_coin));
            spec_choices.copy_live_to(f!((*d).spec_choices));
            f!((*d).last_attack).write(*last_attack);
            fxp.copy_live_to(f!((*d).fxp));
            answers.copy_live_to(f!((*d).answers));
            prompt_payloads.copy_live_to(f!((*d).prompt_payloads));
            f!((*d).payload_serial).write(*payload_serial);
            f!((*d).prevented_events).write(*prevented_events);
            f!((*d).reach_skip).write(*reach_skip);
            f!((*d).survive_groups).write(*survive_groups);
            f!((*d).order_pending).write(*order_pending);
            f!((*d).checkup_held).write(*checkup_held);
            f!((*d).put_by).write(*put_by);
            // The destination keeps its own recording flag (like the chance recording); a recording game restored from a
            // copy keeps the entries made up to the copy (a rolled back action's go).
            let on = (*d).record.on;
            f!((*d).record).write(record.with_on(on));
            if on {
                crate::record::rewind(record.len);
            }
            // The declaration index is fixed per game (built at `start` from the decks): copied only into a game holding
            // another (a fork restored into its own game has it already).
            if (*d).decls.id != decls.id {
                f!((*d).decls).write(*decls);
            }
            // The derived facts are not copied: another line of play may reach the same version numbers.
            (*d).derived.clear();
            // Nor the card lists: another line of play may reach the same layout generation.
            (*d).order_cache.invalidate();
        }
    }

    /// What [`copy_from`](Self::copy_from) moves when `self` is the source (a measurement: `bench --search`,
    /// tests/sizes.rs). Lists the fields as `copy_from` does, so a new field of `Game` must be counted here too.
    /// `#[inline]` (as its helpers `live_bytes` / `clear_bytes`): compiled only where it is called, so the library's code
    /// units stay as they were (a non-inline copy moved the bench's instruction count by +0.5%).
    #[inline]
    pub fn copy_bytes(&self) -> CopyBytes {
        use std::mem::size_of_val as s;
        let Game {
            st, rng, prompts, last_prompt_id, items, waits, fx, temps: _, temp_used, temp_hw, coin_callbacks,
            trial, kinds_present, lock_cards, trace_effects, copy_sessions, copy_serial, copy_chain, deleg, after_dmg, pending, ten_hp, ten_hp_coin, last_attack, spec_choices,
            derived,
            // Copied only into a game holding another game's index (a fork of the same game has it already).
            decls: _,
            order_cache: _,
            fxp,
            answers,
            prompt_payloads,
            payload_serial,
            prevented_events,
            reach_skip,
            survive_groups,
            order_pending,
            checkup_held,
            put_by,
            record,
        } = self;
        let copied = s(st) + s(rng) + prompts.live_bytes() + s(last_prompt_id) + items.live_bytes() + waits.live_bytes()
            + fx.live_bytes()
            + *temp_hw as usize * std::mem::size_of::<List<120>>()
            + s(temp_used) + s(temp_hw) + coin_callbacks.live_bytes() + s(trial) + s(kinds_present) + s(lock_cards)
            + s(trace_effects) + copy_sessions.live_bytes() + s(copy_serial) + copy_chain.live_bytes() + s(deleg)
            + after_dmg.live_bytes() + pending.live_bytes() + ten_hp.live_bytes() + ten_hp_coin.live_bytes()
            + spec_choices.live_bytes() + s(last_attack) + fxp.live_bytes() + answers.live_bytes()
            + prompt_payloads.live_bytes() + s(payload_serial) + s(prevented_events) + s(reach_skip) + s(survive_groups)
            + s(order_pending) + s(checkup_held) + s(put_by) + s(record);
        CopyBytes { copied, cleared: derived.clear_bytes() }
    }
}

/// What [`Game::copy_from`] moves: a restore's memory traffic besides the work after it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CopyBytes {
    /// Copied from the source: the state, the live part of the stacks, the staging lists ever allocated, the fixed
    /// fields (not the declaration index: a restore into a game of the same deal holds it already).
    pub copied: usize,
    /// Written in the destination to forget its derived facts (the memo's stamps).
    pub cleared: usize,
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
            temp_hw: 0,
            coin_callbacks: SVec::new(),
            trial: false,
            kinds_present: crate::effects::KindMask::EMPTY,
            lock_cards: [0; 4],
            trace_effects: false,
            copy_sessions: SVec::new(),
            copy_serial: 0,
            copy_chain: SVec::new(),
            deleg: None,
            after_dmg: SVec::new(),
            pending: SVec::new(),
            ten_hp: SVec::new(),
            ten_hp_coin: SVec::new(),
            last_attack: None,
            spec_choices: SVec::new(),
            derived: crate::derived::Derived::new(),
            decls: crate::decl::DeclIndex::new(),
            order_cache: crate::decl::OrderCache::new(),
            fxp: crate::effects::EffectPayloads::default(),
            answers: SVec::new(),
            prompt_payloads: SVec::new(),
            payload_serial: 0,
            prevented_events: 0,
            reach_skip: 0,
            survive_groups: SVec::new(),
            order_pending: false,
            checkup_held: false,
            put_by: None,
            record: crate::record::Recorder::default(),
        }
    }

    /// Keep a prompt's `MoveOpts` in the payload table (the prompt refers to it).
    pub fn stash_opts(&mut self, o: crate::prompts::MoveOpts) -> crate::prompts::PayloadRef {
        self.stash(crate::prompts::PromptPayload::Opts(o))
    }

    /// Keep a prompt's offered Energy in the payload table.
    pub fn stash_energy(&mut self, e: crate::effects::EnergyMap) -> crate::prompts::PayloadRef {
        self.stash(crate::prompts::PromptPayload::Energy(e))
    }

    fn stash(&mut self, p: crate::prompts::PromptPayload) -> crate::prompts::PayloadRef {
        self.sweep_payloads();
        self.payload_serial = self.payload_serial.wrapping_add(1);
        self.prompt_payloads.push((self.payload_serial, p));
        crate::prompts::PayloadRef(self.payload_serial)
    }

    /// Drop the payloads no prompt refers to.
    fn sweep_payloads(&mut self) {
        let prompts = &self.prompts;
        self.prompt_payloads.retain(|(s, _)| prompts.iter().any(|p| p.kind.payload() == Some(crate::prompts::PayloadRef(*s))));
    }

    /// A prompt's `MoveOpts`.
    pub fn payload_opts(&self, r: crate::prompts::PayloadRef) -> &crate::prompts::MoveOpts {
        match self.prompt_payloads.iter().find(|(s, _)| *s == r.0) {
            Some((_, crate::prompts::PromptPayload::Opts(o))) => o,
            _ => panic!("no MoveOpts payload {:?}", r),
        }
    }

    /// A prompt's offered Energy.
    pub fn payload_energy(&self, r: crate::prompts::PayloadRef) -> &crate::effects::EnergyMap {
        match self.prompt_payloads.iter().find(|(s, _)| *s == r.0) {
            Some((_, crate::prompts::PromptPayload::Energy(e))) => e,
            _ => panic!("no Energy payload {:?}", r),
        }
    }

    /// The cards of declaration family `f` in place, in the propagation order of rank class `class`
    /// ([`crate::decl::in_place`] through the order cache).
    #[inline]
    pub fn decl_in_place(&mut self, f: usize, class: u16) -> SVec<CardId, 120> {
        let r = crate::decl::in_place_cached(&self.st, &self.decls, &mut self.order_cache, None, f, class);
        if verify_cache() {
            assert_eq!(r.as_slice(), crate::decl::in_place(self, f, class).as_slice(), "the order cache's in-place list differs (family {f})");
        }
        r
    }

    /// The cards of `set` in the propagation order of rank class `class` ([`crate::decl::kind_order`] through the order
    /// cache).
    #[inline]
    pub fn decl_ordered(&mut self, set: u128, class: u16) -> SVec<CardId, 120> {
        let r = crate::decl::ordered_cached(&self.st, &mut self.order_cache, None, set, class);
        if verify_cache() {
            assert_eq!(r.as_slice(), crate::decl::kind_order(self, set, class).as_slice(), "the order cache's list differs (class {class})");
        }
        r
    }

    /// The cards of family `f` in any zone, in the propagation order of rank class `class` ([`crate::decl::all`]).
    #[inline]
    pub fn decl_all(&mut self, f: usize, class: u16) -> SVec<CardId, 120> {
        let set = self.decls.sets[f];
        self.decl_ordered(set, class)
    }

    /// [`crate::decl::kind_listeners`] through the order cache.
    #[inline]
    pub fn decl_kind_listeners(&mut self, kind: u32) -> SVec<CardId, 120> {
        let set = self.decls.kind(kind);
        let r = self.decl_ordered(set, 0);
        if verify_cache() {
            assert_eq!(r.as_slice(), crate::decl::kind_listeners(self, kind).as_slice(), "the order cache's listeners differ (kind {kind})");
        }
        r
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

    /// Opens the attack's after-damage and trigger windows.
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
        crate::engine::order::drop_window(self, crate::engine::order::Tag::Damaged(atk));
    }

    /// There are step 7 triggers of the attack left to resolve.
    pub fn attack_triggers_pending(&self, atk: EffId) -> bool {
        crate::engine::order::pending(self, crate::engine::order::Tag::Damaged(atk))
    }

    /// `RESOLVE_NEXT_ATTACK_TRIGGER` (APR E-03, "the player of the Pokémon taking the damage gets to choose in which
    /// order to apply those effects"): the next step of the attack's window, in the order recorded unless two or more
    /// distinct effects apply (`engine::order`). The caller waits for prompts between calls.
    pub fn resolve_next_attack_trigger(&mut self, atk: EffId) -> R {
        let mut w = crate::engine::order::Window::Damaged { atk };
        crate::engine::order::step(self, &mut w).map(|_| ())
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
                self.after_dmg.remove_at(i);
            } else {
                i += 1;
            }
        }
    }

    /// Closes the attack's after-damage window, then run the queued steps in order.
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
                AfterDmgStep::LeaveCards { target, cards, zone, cause } => {
                    crate::engine::knockout::leave_play_cards(self, target, cards.as_slice(), zone, cause, None)?;
                }
                AfterDmgStep::MoveAttached { card, from, to, cause } => {
                    crate::engine::attach::move_attached(self, card, from, to, cause)?;
                }
                AfterDmgStep::Shuffle(p) => crate::prefabs::shuffle_deck(self, p as usize),
            }
        }
        Ok(())
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

    /// A coin flip of player `p` for a card's effect (`cause`), whose result goes to `cb` (`engine::condition::flip_coin`:
    /// a CoinFlip event, a re-flip offered when a declaration covers it).
    pub fn coin_flip(&mut self, p: usize, cb: CoinCb, cause: crate::cause::Cause) -> R<Option<bool>> {
        let cb = self.tag_coin(cb);
        crate::engine::condition::flip_coin(self, p, cb, cause).map(Some)
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
                self.temp_hw = self.temp_hw.max(i as u8 + 1);
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

    /// Ask `player` (`kind`, for `purpose`; `provenance`: the card whose effect asks and the card the question is about),
    /// then `cont` with the answer. A chance or information prompt has no provenance (`Provenance::NONE`).
    pub fn prompt(&mut self, player: usize, purpose: PromptPurpose, kind: PromptKind, provenance: Provenance, cont: Cont) {
        if verify_cache() {
            // A chance or information prompt asks no question; a card program's question is its card's.
            assert!(provenance == Provenance::NONE || kind.class() == PromptClass::Decision, "a chance or information prompt asks no question");
            if let (Cont::Card { card, .. }, PromptClass::Decision) = (cont, kind.class()) {
                assert_eq!(provenance.source, Some(card), "card {card}'s program asks a question that isn't its own");
            }
        }
        let kind = construct_prompt(self, kind);
        let cont = self.tag_cont(cont);
        let id = self.next_id();
        // Safety cap: a card duplicating itself in a loop would get here (the Dangle Tail aliasing that
        // used to is fixed, W1-E; there are no approved divergences for it any more).
        assert!(self.prompts.len() < self.prompts.capacity() && self.items.len() < self.items.capacity(), "prompt stack exhausted");
        self.prompts.push(PromptRec { id, player: player as u8, purpose, kind, provenance, answered: false, partial: false });
        let mut ids = SVec::new();
        ids.push(id);
        // A question with no answer it can take isn't asked, and the op that asks it does nothing (batch 10 E19, the
        // companion of RULES.md "Options that can't do anything": no option offered does nothing). It keeps its place
        // among the prompts as an information prompt, and its continuation gets no answer.
        let unasked = kind.class() == PromptClass::Decision && !self.has_an_answer(self.prompts.len() - 1);
        if unasked {
            let pr = self.prompts.as_mut_slice().last_mut().expect("just pushed");
            (pr.purpose, pr.kind, pr.provenance) = (PromptPurpose::Wait, PromptKind::Wait, Provenance::NONE);
        } else {
            crate::expect::on_prompt(self, &kind, matches!(cont, Cont::CoinFlipWait { .. }));
        }
        self.items.push(PromptItem { ids, cont, unasked });
    }

    /// Several prompts answered together, then `cont` (the setup's shuffles, a Pokémon's Sleep flips): chance prompts,
    /// so no provenance.
    pub fn prompt_group(&mut self, prompts: &[(usize, PromptPurpose, PromptKind)], cont: Cont) {
        let cont = self.tag_cont(cont);
        let mut ids = SVec::new();
        for &(player, purpose, kind) in prompts {
            let id = self.next_id();
            let kind = construct_prompt(self, kind);
            self.prompts.push(PromptRec { id, player: player as u8, purpose, kind, provenance: Provenance::NONE, answered: false, partial: false });
            ids.push(id);
        }
        self.items.push(PromptItem { ids, cont, unasked: false });
    }

    pub fn wait_prompt(&mut self, cont: Cont) {
        let cont = self.tag_cont(cont);
        self.waits.push(cont);
    }

    /// Continuations created while delegated source code runs belong to the
    /// source's card, so they resume in the source's code.
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

    /// A silent wait prompt with a continuation.
    pub fn wait(&mut self, player: usize, cont: Cont) {
        self.prompt(player, PromptPurpose::Wait, PromptKind::Wait, Provenance::NONE, cont);
    }

    fn resolve_wait_items(&mut self) -> R {
        while self.items.is_empty() && !self.waits.is_empty() {
            let c = self.waits.pop().unwrap();
            self.run_cont(c, &[])?;
        }
        Ok(())
    }

    /// Resolves the prompt `id` with `result` and runs its group's continuation once every prompt of the group is answered.
    fn reduce_prompt(&mut self, id: u32, result: Res) -> R {
        let pi = match self.prompts.iter().position(|p| p.id == id) {
            Some(i) => i,
            None => return Ok(()),
        };
        let ii = match self.items.iter().position(|it| it.ids.contains(&id)) {
            Some(i) => i,
            None => return Ok(()),
        };
        if self.prompts.as_slice()[pi].answered {
            bail!("PROMPT_ALREADY_RESOLVED");
        }
        self.prompts.as_mut_slice()[pi].answered = true;
        self.answers.push((id, result));
        if self.record.on && matches!(self.prompts.as_slice()[pi].kind, PromptKind::ShuffleDeck) {
            crate::record::shuffle(self, self.prompts.as_slice()[pi].player as usize);
        }
        let item = self.items.as_slice()[ii];
        let all = item.ids.iter().all(|pid| {
            self.prompts.iter().find(|p| p.id == *pid).map(|p| p.answered).unwrap_or(false)
        });
        let r: R = (|| {
            if all {
                let mut results: SVec<Res, 4> = SVec::new();
                for pid in item.ids.iter() {
                    if self.prompts.iter().any(|p| p.id == *pid) {
                        if let Some((_, a)) = self.answers.iter().find(|(x, _)| x == pid) {
                            results.push(*a);
                        }
                    }
                }
                // Resolved prompts are no longer needed.
                self.prompts.retain(|p| !item.ids.contains(&p.id));
                self.answers.retain(|(x, _)| !item.ids.contains(x));
                // The continuation runs while the item is still queued and the item is removed afterwards, at the
                // index found beforehand (a continuation may queue prompts of its own).
                self.run_cont(item.cont, if item.unasked { &[] } else { results.as_slice() })?;
                self.items.remove_at(ii);
            }
            self.resolve_wait_items()
        })();
        if r.is_err() {
            self.prompts.retain(|p| !item.ids.contains(&p.id));
            self.answers.retain(|(x, _)| !item.ids.contains(x));
            if let Some(idx) = self.items.iter().position(|it| it.ids.contains(&id)) {
                self.items.remove_at(idx);
            }
        }
        r
    }

    fn run_cont(&mut self, c: Cont, results: &[Res]) -> R {
        self.st.bump();
        let first = results.first().copied().unwrap_or(Res::Null);
        match c {
            Cont::Noop => Ok(()),
            Cont::Setup(f) => setup::resume(self, f, results),
            Cont::PhasePlayerTurn => {
                self.st.phase = GamePhase::PlayerTurn;
                self.st.bump();
                Ok(())
            }
            Cont::CheckState(f) => crate::engine::knockout::resume(self, f),
            Cont::TakePrizes { p, cause } => crate::engine::knockout::take_prizes_cont(self, p, cause, first),
            Cont::OrderEffects { w } => crate::engine::order::chosen(self, w, first.as_int()),
            Cont::OrderNext { w } => crate::engine::order::next(self, w),
            Cont::SurviveOrder { grp } => crate::engine::damage::survive_order_chosen(self, grp, first.as_int()),
            Cont::WhatIfOver => crate::engine::knockout::state_check(self, OnComplete::None),
            Cont::DiscardAttackerEnergy { owner, attack, source_card, target } => crate::engine::game_effect::discard_attacker_energy_cont(self, owner, attack, source_card, target, first),
            Cont::ChooseActive { p } => crate::engine::knockout::choose_active_cont(self, p, first),
            Cont::BenchShrink { p, empty } => crate::engine::knockout::bench_shrink_cont(self, p, empty, first),
            Cont::BetweenTurnsWait { oc } => phase::run_between_turns_effects(self, oc),
            Cont::BetweenTurnsCheck { oc } => crate::engine::knockout::state_check(self, oc),
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
            Cont::Reflip(a) => crate::engine::condition::reflip_answer(self, a, first.as_bool()),
            Cont::TrainerCleanup { p, card } => crate::engine::play_trainer::trainer_cleanup(self, p as usize, card),
            Cont::ShuffleApplyNoWait { p } => {
                if let Res::Order(o) = first {
                    apply_order(&mut self.st.zone_edit(ListRef::Deck(p)), o.as_slice());
                    self.st.bump_layout();
                }
                Ok(())
            }
            Cont::ShuffleApply { p } => {
                if let Res::Order(o) = first {
                    apply_order(&mut self.st.zone_edit(ListRef::Deck(p)), o.as_slice());
                    self.st.bump_layout();
                }
                // SHUFFLE_DECK's silent wait prompt.
                self.wait(p as usize, Cont::Noop);
                Ok(())
            }
            Cont::Prefab(c) => crate::prefabs::resume(self, c, results),
            Cont::SuddenDeathCoin => {
                // The flip for who goes first in the Sudden Death game (player 1 flips; heads, they go first).
                crate::engine::condition::coin_flipped(self, 0, crate::spec::event::CoinPurpose::FirstPlayer, first.as_bool(), crate::cause::Cause::rule(crate::cause::RuleWhich::Setup, 0))?;
                crate::engine::knockout::setup_sudden_death_game(self, if first.as_bool() { 0 } else { 1 })
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
            CoinCb::PlayGate { p, card, target } => crate::engine::play_trainer::gate_coin(self, p, card, target, result),
            CoinCb::Reflip { p, callback, cause, offer } => {
                crate::engine::condition::ask_reflip(self, crate::engine::condition::ReflipAsk { p, callback, cause, offer, result, results: 0, n: 0, mode: None });
                Ok(())
            }
            CoinCb::Sequence { p, mode, results, n, callback, cause, offer } => {
                let results = if result { results | (1 << n) } else { results };
                let n = n + 1;
                let more = if mode == 0 { result } else { n < mode };
                if more {
                    let cb = CoinCb::Sequence { p, mode, results, n, callback, cause, offer };
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
                    let _ = k;
                    crate::engine::condition::flip_raw(self, p as usize, cb, cause)?;
                    return Ok(());
                }
                // The re-flip offer, its declaration asked again now the flips are done.
                if offer {
                    if let Some(offer) = crate::engine::condition::reflip_offered(self, p as usize, cause)? {
                        crate::engine::condition::ask_reflip(self, crate::engine::condition::ReflipAsk { p, callback, cause, offer, result, results, n, mode: Some(mode) });
                        return Ok(());
                    }
                }
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

    /// Cards with a handler for this effect kind, in propagation order (zone order,
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
            for c in pl.stadium().iter() {
                add(c, &mut cards);
            }
            for c in pl.supporter().iter() {
                add(c, &mut cards);
            }
            let a = &pl.slots[pl.active as usize];
            for c in a.cards().iter() {
                add(c, &mut cards);
            }
            for c in a.tools().iter() {
                add(c, &mut cards);
            }
            for &b in pl.bench.iter() {
                let s = &pl.slots[b as usize];
                for c in s.cards().iter() {
                    add(c, &mut cards);
                }
                for c in s.tools().iter() {
                    add(c, &mut cards);
                }
            }
            for i in 0..pl.prize_count as usize {
                for c in pl.prizes()[i].iter() {
                    add(c, &mut cards);
                }
            }
            for c in pl.hand().iter() {
                add(c, &mut cards);
            }
            for c in pl.deck().iter() {
                add(c, &mut cards);
            }
            for c in pl.discard().iter() {
                add(c, &mut cards);
            }
        }
        if cards.len() < 2 {
            return cards;
        }
        cards.as_mut_slice().sort_by_key(|c| prop_rank(self, class, *c)); // stable
        cards
    }

    /// Reduces one effect: card handlers in propagation order, then the rule's own reducer.
    #[inline]
    pub fn reduce_effect(&mut self, id: EffId) -> R {
        if self.record.on {
            return crate::record::effect(self, id);
        }
        self.reduce_effect_now(id)
    }

    /// [`Game::reduce_effect`] itself (the game record opens the effect's event around it).
    pub(crate) fn reduce_effect_now(&mut self, id: EffId) -> R {
        // The use of a power with no program writes nothing a fact reads: the derived facts stay fresh through it.
        let read = match *self.e(id) {
            Effect::Power { power, .. } => !crate::spec::run::power_has_program(self, power),
            _ => false,
        };
        if !read {
            self.st.bump();
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
        // the copycat can't use or probe an Ability of the copycat.
        if let Effect::Power { card, .. } = *self.e(id) {
            if matches!(self.deleg, Some(d) if d.attacks && d.copycat == card) {
                return Err(GameError("BLOCKED_BY_EFFECT"));
            }
        }

        // The cards the effect reaches (batch 8 C5: no handler walk over every card that lists the kind): the acting card's
        // program and the declarations the effect is read by, in the propagation order.
        let kind = self.e(id).kind();
        let class = prop_class(self.e(id));
        let order = self.handler_cards(id, kind, class);
        for &c in order.iter() {
            // A card's own handler never runs as a delegate.
            let saved = self.deleg.take();
            let r = crate::spec::run::reduce(self, c, id);
            self.deleg = saved;
            r?;
            if !read {
                // A card's handler may write (records, flags): the next one reads fresh facts.
                self.st.bump();
            }
        }
        // The effects cards put for a time ("this turn, attacks used by this Pokémon do N more damage"), after the cards' handlers.
        if !self.st.lasting.is_empty() && kind == k::ATTACK {
            crate::spec::passive::declared_attack_stage(self, id)?;
        }
        // A copied attack being used: its text, the source card's code run as the copycat, after the game's cards.
        if !self.copy_sessions.is_empty() {
            crate::copy_attack::resolve_sessions(self, id)?;
        }

        if self.prevented(id) {
            return Ok(());
        }

        if !read {
            // The handlers ran (they may write); the reducers below write the event's consequences.
            self.st.bump();
        }
        if self.record.on {
            crate::record::before_consequences(self, id);
        }
        // The effect's consequences: one reducer per kind.
        match kind {
            k::END_TURN | k::AFTER_ATTACK | k::BETWEEN_TURNS => phase::reducer(self, id)?,
            k::GAIN_CONDITION | k::REMOVE_CONDITION | k::HEAL | k::COIN_FLIP => crate::engine::condition::reducer(self, id)?,
            k::ATTACH | k::MOVE_ENERGY | k::MOVE_TOOL => crate::engine::attach::reducer(self, id)?,
            k::CHANGE_ACTIVE => crate::engine::change_active::reducer(self, id)?,
            k::PLACE_COUNTERS | k::MOVE_COUNTERS_EVENT | k::DAMAGE => crate::engine::damage::reducer(self, id)?,
            k::DISCARD | k::PUT_INTO_HAND | k::PUT_INTO_DECK | k::DRAW => crate::engine::cards_zone::reducer(self, id)?,
            k::KNOCK_OUT | k::LEAVE_PLAY | k::TAKE_PRIZES => crate::engine::knockout::reducer(self, id)?,
            k::APPLY_EFFECT => crate::engine::apply::reducer(self, id)?,
            k::DEVOLVE | k::SWAP | k::ENTER_PLAY | k::EVOLVE => crate::engine::enter::reducer(self, id)?,
            k::PLAY_TRAINER => crate::engine::play_trainer::reducer(self, id)?,
            k::RETREAT => retreat::reducer(self, id)?,
            k::USE_ATTACK | k::USE_POWER | k::USE_STADIUM => crate::engine::game_effect::reducer(self, id)?,
            _ => {}
        }
        if matches!(
            kind,
            k::DISCARD
                | k::PUT_INTO_HAND
                | k::PUT_INTO_DECK
                | k::DRAW
                | k::ENTER_PLAY
                | k::EVOLVE
                | k::DEVOLVE
                | k::CHANGE_ACTIVE
        ) && !(kind == k::ENTER_PLAY && self.st.phase == GamePhase::Setup)
        {
            // The Pokémon put down at setup take hold together when setup ends (`setup::finish`).
            crate::spec::passive::lock_sync(self);
        } else if matches!(kind, k::ATTACH | k::MOVE_ENERGY | k::MOVE_TOOL) {
            // Only when an attached card can turn a lock source's Ability on or off.
            crate::spec::passive::lock_sync_attached(self);
        }
        if !read {
            self.st.bump();
        }
        if crate::spec::event::TRIGGER_EVENT_KINDS.has(kind) {
            crate::spec::run::after_event(self, id)?;
        }
        // The damaged Pokémon's own lasting trigger, after the cards' triggers over the Damage (APR E-03).
        if kind == k::DAMAGE {
            crate::engine::damage::record_retaliation(self, id)?;
        }
        Ok(())
    }

    /// The cards effect `id` (of kind `kind`, rank class `class`) reaches, in the propagation order: the acting card's
    /// program (the attack's, the Ability's, the Trainer's, the Stadium's) and the declarations its kind is read by (the Attack-stage
    /// damage bonuses, the next-turn bonus and Shred on an `Attack`; the Ability locks on a `Power`; the end-of-turn,
    /// Checkup and first-turn handlers; the check effects of the slow path). An event the routines
    /// and the triggers after it read reaches no card (its declarations are read through `decl::kind_listeners`).
    fn handler_cards(&mut self, id: EffId, kind: u32, class: u16) -> SVec<CardId, 120> {
        if !self.kinds_present.has(kind) {
            return SVec::new();
        }
        let all = self.decls.kind(kind);
        if all == 0 {
            return SVec::new();
        }
        let bit = |c: CardId| if (c as usize) < MAX_CARDS { 1u128 << c } else { 0 };
        let locks = (self.lock_cards[0] as u128) | (self.lock_cards[1] as u128) << 64;
        let set = match *self.e(id) {
            Effect::Attack { attack, .. } => (bit(attack.card) | self.decls.attack_passives) & all,
            Effect::AfterAttack { attack, .. } => bit(attack.card) & all,
            Effect::Power { card, .. } => (bit(card) | locks) & all,
            Effect::PlayTrainer { card, .. } => bit(card) & all,
            Effect::UseStadium { stadium, .. } => bit(stadium) & all,
            // Read by their routines and by the triggers after the event, never by a handler (`decl::UNHANDLED_KINDS`).
            _ if crate::decl::UNHANDLED_KINDS.has(kind) => 0,
            _ => all,
        };
        let out = self.decl_ordered(set, class);
        if verify_cache() {
            // A subsequence of the broadcast (every card of the game whose handler mask has the kind), in its order.
            let slow = self.propagation_order_slow(class, kind);
            let mut it = slow.iter();
            assert!(out.iter().all(|c| it.any(|x| x == c)), "the cards effect kind {kind} reaches aren't in the broadcast's order");
        }
        out
    }

    // -----------------------------------------------------------------------
    // Dispatch

    /// Adds both players with their decks; starts setup.
    pub fn start(&mut self, decks: [&[crate::carddb::DefId]; 2]) -> R {
        for (p, deck) in decks.iter().enumerate() {
            let base = self.st.n_cards;
            for (i, d) in deck.iter().enumerate() {
                let c = base + i as u8;
                self.st.cards[c as usize] = CardInst { def: *d, owner: p as u8, ..Default::default() };
                self.st.zone_edit(ListRef::Deck(p as u8)).push(c);
            }
            self.st.n_cards += deck.len() as u8;
            if !setup::deck_is_valid(&self.st, deck) {
                self.st.players_added = p as u8;
                self.st.phase = GamePhase::Finished;
                self.st.winner = WINNER_NONE;
                return Ok(());
            }
        }
        self.st.phase = GamePhase::Setup;
        self.index_cards();
        self.st.bump_layout();
        setup::start(self)?;
        self.after_dispatch()
    }

    /// Build the per-card indexes from the cards' definitions: the effect kinds present (`kinds_present`), the cards
    /// declaring an Ability lock (`lock_cards`) and the declaration index (`decls`, with a new identity so `copy_from`
    /// carries it into the game's forks). By `start`, and again when a card's identity is rewritten
    /// (`Game::set_unseen`).
    pub(crate) fn index_cards(&mut self) {
        self.kinds_present = crate::effects::KindMask::EMPTY;
        self.lock_cards = [0; 4];
        for c in 0..self.st.n_cards {
            let d = self.st.cards[c as usize].def;
            if cards::spec_for(d).map_or(false, |s| crate::spec::passive::declares_ability_lock(s.passives)) {
                self.lock_cards[(c >> 6) as usize] |= 1u64 << (c & 63);
            }
            if let Some(imp) = cards::impl_for(d) {
                self.kinds_present |= imp.mask;
            }
        }
        self.decls = crate::decl::DeclIndex::build(self);
    }

    fn after_dispatch(&mut self) -> R {
        self.st.bump();
        self.resolve_wait_items()?;
        if self.items.is_empty() {
            crate::engine::knockout::state_check(self, OnComplete::None)?;
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

    /// Resolves a prompt with a decoded result.
    pub fn resolve(&mut self, prompt_index: usize, result: Res) -> R {
        let id = self.prompts.as_slice()[prompt_index].id;
        self.st.bump();
        let r = self.reduce_prompt(id, result);
        if r.is_ok() && self.items.is_empty() {
            self.st.bump();
            let r2 = crate::engine::knockout::state_check(self, OnComplete::None);
            self.gc();
            return r2;
        }
        self.gc();
        r
    }

    /// A turn action on a scratch copy the caller discards on error (no backup).
    pub fn act_trial(&mut self, action: Action) -> R {
        if self.prompts.iter().any(|p| !p.answered) {
            bail!("ACTION_IN_PROGRESS");
        }
        self.items.clear();
        self.st.bump();
        turn::play_card_reducer(self, action)?;
        turn::player_turn_reducer(self, action)?;
        self.after_dispatch()
    }

    /// A turn action: all-or-nothing.
    pub fn act(&mut self, action: Action) -> R {
        self.act_with(action, true)
    }

    /// [`Game::act`] for a driver that gives up the game on an error (self-play, the benchmark): no backup
    /// copy, so a rejected action leaves the game half done.
    pub fn act_no_rollback(&mut self, action: Action) -> R {
        self.act_with(action, false)
    }

    fn act_with(&mut self, action: Action, rollback: bool) -> R {
        if self.prompts.iter().any(|p| !p.answered) {
            bail!("ACTION_IN_PROGRESS");
        }
        self.st.bump();
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
            if p.answered {
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

    /// Chance outcome drawn from the internal RNG.
    pub fn draw_chance(&mut self, i: usize) -> Res {
        let pr = self.prompts.as_slice()[i];
        match pr.kind {
            PromptKind::ShuffleDeck => {
                let p = pr.player as usize;
                let n = self.st.players[p].deck().len();
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

}

/// Reorders a list: ignored unless `order` is a permutation of the list.
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
