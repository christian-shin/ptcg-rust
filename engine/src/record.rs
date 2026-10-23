//! The game record: an opt-in list of what happened in a game, one entry per completed event, with who can see it.
//!
//! A host that shows a player what happened (a log, an opponent model's evidence) reads the record instead of diffing
//! states. It is off by default ([`Game::record_on`]); while off every event pays one predictable branch and nothing
//! else. While on, the entries go into a buffer of the calling thread (like the chance outcomes of `rng`: one recording
//! game per thread at a time) and [`Game::take_record`] hands them over.
//!
//! What is recorded, in the order it happens:
//! - every card that moves, one entry per card (`from` and `to` are [`CardPlace`]s, `to` read when the event that moved
//!   it is done), tagged with the event that moved it: Discard, PutIntoHand, PutIntoDeck, Draw, LeavePlay, TakePrizes,
//!   EnterPlay, Evolve, Devolve, Swap, Attach, MoveEnergy, MoveTool; and the routines no card reacts to (user decision
//!   D2 of events batch 7): Look (cards set aside), Mulligan, SetPrizes; the Trainer card's way into and out of the play
//!   area (PlayTrainer);
//! - one entry per event that moves no card: BeginTurn, EndTurn, Checkup, UseAttack, UseAbility, UseStadium, Retreat,
//!   PlayTrainer (before its effect's entries), AttackDone, Damage, PlaceCounters, MoveCounters (one per pair),
//!   RemoveCounters, GainCondition, RemoveCondition, CoinFlip (with its purpose and result), KnockOut, ApplyEffect;
//!   ChangeActive as two entries (the Pokémon coming in, the one going to the Bench) whose places are their spots;
//! - Shuffle (one per deck shuffled), Reveal (one per card shown: a hand revealed for a mulligan, cards shown to a
//!   player, the Pokémon put down face down at setup turned face up), the setup check for a Basic Pokémon (a Mulligan
//!   entry with [`Detail::BasicCheck`]) and GameEnd.
//!
//! - a damage, damage counters or a heal that comes out 0 (RULES.md "Events batch 10, E14": no event happens, nothing
//!   reacts, nothing changes) has a record-only entry of the event's kind with amount 0 ([`comes_out_zero`]), for a host
//!   that shows it: an attack's damage that the calculation brings to 0 or that a prevention stops (the base damage of
//!   an attack with printed damage brought to 0, "takes N less", "prevent all damage", Dig), damage counters a lock or a
//!   prevention keeps off a Pokémon, a heal that removes nothing (no damage to heal, nothing to heal by, "can't be
//!   healed").
//!
//! Any other event that doesn't happen (refused by its checks, prevented) records nothing; a turn action or an answer that
//! fails and is rolled back takes its entries with it (`Game::copy_from` rewinds the buffer). Legality trials and
//! every other copy made by `Game::fork` never record.
//!
//! Who can see an entry (`seen_by`) is the rules' judgment: a card's identity is seen by whoever sees it where it was
//! or where it went: the deck and face-down Prize cards by nobody, a hand by its owner, the discard pile, the Lost Zone,
//! play, the Stadium and the play area by everyone (but the Pokémon put down face down at setup only by their owner
//! until they are turned face up: SSP rulebook p. 8, steps 5, 6 and 8), a card set aside for a look by the player whose
//! effect looks at it when it is set aside (and by everyone when it came from a public zone; a card set aside without
//! being looked at, the shuffled Prize cards of a redeal, by nobody). Every other entry is public.

use crate::cause::{Cause, RuleWhich};
use crate::effects::{ApplyTarget, EffId, Effect, PowerRef};
use crate::engine::knockout::EndReason;
use crate::game::Game;
use crate::list::{CardId, CardList};
use crate::place::{CardPlace, Zone};
use crate::spec::event::{CoinPurpose, EventKind, RulesZone, TrainerUse};
use crate::state::{AttackRef, ListRef, NO_PLAYER};
use crate::types::{SpecialCondition, Winner};

/// What an entry says beyond its kind, cards, places and amount.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    None,
    /// CoinFlip: what the coin was flipped for, and the result.
    Coin { purpose: CoinPurpose, heads: bool },
    /// GainCondition / RemoveCondition: the Special Condition.
    Condition(SpecialCondition),
    /// UseAttack, AttackDone: the attack.
    Attack(AttackRef),
    /// UseAbility: the power used.
    Power(PowerRef),
    /// PlayTrainer: played from the hand, or its effect used by another card.
    Trainer(TrainerUse),
    /// MoveCounters: the HP of damage counters taken off the first Pokémon and put on the second (fewer when a prevention
    /// keeps them off it).
    Counters { removed: i32, placed: i32 },
    /// Mulligan: the setup check of a player's hand (SSP rulebook p. 8 step 4): whether it holds a Basic Pokémon (or a
    /// card that may be put down at setup in its place); none means a mulligan follows.
    BasicCheck { found: bool },
    /// GameEnd: the winner (`types::WINNER_*`) and why; a tie that starts a Tiebreaker game has no reason (APR I-E).
    End { winner: Winner, reason: Option<EndReason> },
}

/// One thing that happened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecordEntry {
    pub kind: EventKind,
    /// The player the event is of: the owner of the cards that move or of the Pokémon it is done to, the player whose
    /// turn begins or ends, who flips, whose deck is shuffled, who is shown a card; `NO_PLAYER` for the end of the game.
    pub player: u8,
    /// What made it happen (`None` for the turn structure and the routines no cause is given to).
    pub cause: Option<Cause>,
    /// The card that moves, is played, used or shown; the Pokémon an event is done to (Damage, KnockOut, ...).
    pub card: Option<CardId>,
    /// For a card that moves: where it was, and where it is once the event is done.
    pub from: Option<CardPlace>,
    pub to: Option<CardPlace>,
    /// The Pokémon the event is done to or the card goes onto (its top card): the Pokémon evolved, devolved, attached
    /// to, damaged, healed, Knocked Out, left by a card leaving play, the one counters or an attached card move onto.
    pub target: Option<CardId>,
    /// The other Pokémon of a move between two: the one an attached card or counters leave, the one a Swap replaces, the
    /// other Pokémon of a ChangeActive.
    pub other: Option<CardId>,
    /// Damage, damage counters (HP), the HP healed; 0 for one that comes out 0 ([`comes_out_zero`]: no event happened).
    pub amount: i32,
    pub detail: Detail,
    /// Who sees what the entry says (the card's identity for a card entry), by player index.
    pub seen_by: [bool; 2],
}

/// The record's state in a game: whether it records, the event being recorded, and how many entries it has recorded
/// (the thread's buffer holds exactly those; `Game::copy_from` rewinds to the copy's count).
#[derive(Clone, Copy, Debug, Default)]
pub struct Recorder {
    pub(crate) on: bool,
    ctx: Option<Ctx>,
    pub(crate) len: u32,
}

/// The event whose moves are being recorded.
#[derive(Clone, Copy, Debug)]
struct Ctx {
    kind: EventKind,
    player: u8,
    cause: Option<Cause>,
    /// `RecordEntry::target` / `other` of the moves made while it is open.
    target: Option<CardId>,
    other: Option<CardId>,
    /// The first entry made while it is open.
    start: u32,
    /// The dispatched effect it records, and the entry kept for it (before the entries of what its handlers do).
    eff: Option<EffId>,
    held: Option<u32>,
    filled: bool,
}

#[derive(Default)]
struct Buf {
    entries: Vec<RecordEntry>,
    /// The card entries whose `to` is read when their event is done.
    pending: Vec<u32>,
    /// Moves made with no event open (an engine defect; `PTCG_VERIFY_CACHE=1` panics on one).
    unattributed: u32,
}

thread_local! {
    static BUF: std::cell::RefCell<Buf> = std::cell::RefCell::new(Buf::default());
}

impl Recorder {
    /// Whether the game records.
    #[inline]
    pub fn on(&self) -> bool {
        self.on
    }

    /// This state with the recording flag `on` (`Game::copy_from`: the destination keeps its own).
    #[inline]
    pub(crate) fn with_on(self, on: bool) -> Recorder {
        Recorder { on, ..self }
    }
}

impl Game {
    /// Record this game from now on (call before `start` to record the setup). Clears the thread's record buffer.
    pub fn record_on(&mut self) {
        BUF.with(|b| *b.borrow_mut() = Buf::default());
        self.record = Recorder { on: true, ctx: None, len: 0 };
    }

    /// Stop recording; the entries not taken are dropped.
    pub fn record_off(&mut self) {
        if self.record.on {
            BUF.with(|b| *b.borrow_mut() = Buf::default());
        }
        self.record = Recorder::default();
    }

    /// The entries recorded since the last call (between actions and answers: every event is done then).
    pub fn take_record(&mut self) -> Vec<RecordEntry> {
        let mut out = Vec::new();
        self.take_record_into(&mut out);
        out
    }

    /// [`Game::take_record`] into `out` (cleared first): the entries trade buffers with `out`, so a caller that keeps
    /// one `out` allocates no buffer per call.
    pub fn take_record_into(&mut self, out: &mut Vec<RecordEntry>) {
        out.clear();
        if !self.record.on {
            return;
        }
        self.record.len = 0;
        BUF.with(|b| {
            let mut b = b.borrow_mut();
            b.pending.clear();
            std::mem::swap(&mut b.entries, out);
        })
    }

    /// A copy that doesn't record (`Game::fork`).
    #[inline]
    pub(crate) fn record_stop_in_copy(&mut self) {
        self.record.on = false;
    }
}

/// `Game::copy_from` into a recording game: the buffer goes back to the copy's count (a rolled back action's entries
/// go with it).
pub(crate) fn rewind(len: u32) {
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        b.entries.truncate(len as usize);
        b.pending.retain(|i| *i < len);
    });
}

/// Moves made with no event open since the buffer was cleared (tests).
#[cfg(test)]
fn unattributed() -> u32 {
    BUF.with(|b| b.borrow().unattributed)
}

fn push(g: &mut Game, e: RecordEntry, pending: bool) -> u32 {
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        let i = b.entries.len() as u32;
        b.entries.push(e);
        if pending {
            b.pending.push(i);
        }
        g.record.len = b.entries.len() as u32;
        i
    })
}

/// Insert `e` at `at` (after a held entry: its event's other entries).
fn insert(g: &mut Game, at: u32, e: RecordEntry, pending: bool) {
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        b.entries.insert(at as usize, e);
        for i in b.pending.iter_mut() {
            if *i >= at {
                *i += 1;
            }
        }
        if pending {
            b.pending.push(at);
        }
        g.record.len = b.entries.len() as u32;
    });
}

fn remove(g: &mut Game, at: u32) {
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        b.entries.remove(at as usize);
        b.pending.retain(|i| *i != at);
        for i in b.pending.iter_mut() {
            if *i > at {
                *i -= 1;
            }
        }
        g.record.len = b.entries.len() as u32;
    });
}

const ALL: [bool; 2] = [true, true];
const NOBODY: [bool; 2] = [false, false];

fn only(p: u8) -> [bool; 2] {
    let mut s = NOBODY;
    if (p as usize) < 2 {
        s[p as usize] = true;
    }
    s
}

fn or(a: [bool; 2], b: [bool; 2]) -> [bool; 2] {
    [a[0] || b[0], a[1] || b[1]]
}

fn public(z: RulesZone) -> bool {
    matches!(z, RulesZone::Discard | RulesZone::LostZone | RulesZone::InPlay | RulesZone::Stadium)
}

/// Who sees a card of `owner` set aside for a look by player `by`, from the zone `from`: the player looking, and everyone
/// when it came from a public zone; its owner only if they are the one looking, or when the table shows a look to the
/// card's owner (`TableRules::looked_cards_shown_to_owner`). The one rule: the record's Look entry and
/// `TableView::looking` both read it.
pub fn look_seen_by(rules: &crate::table_rules::TableRules, from: RulesZone, by: u8, owner: u8) -> [bool; 2] {
    if public(from) {
        ALL
    } else if rules.looked_cards_shown_to_owner {
        or(only(by), only(owner))
    } else {
        only(by)
    }
}

/// Who sees card `c` in `place`, for an entry of `kind`: a card set aside for a look is seen by the player looking
/// (`by`) when it is set aside (the Look), by nobody else unless it came from a public zone.
fn sees(g: &Game, c: CardId, place: Option<CardPlace>, kind: EventKind, by: u8) -> [bool; 2] {
    let Some(place) = place else { return NOBODY };
    match place.zone {
        Zone::Deck => NOBODY,
        Zone::Hand => only(place.owner),
        Zone::Prizes => {
            let pl = &g.st.players[place.owner as usize];
            let up = (0..pl.prize_count as usize).any(|i| pl.prize_face_up[i] && pl.prizes()[i].contains(c));
            if up {
                ALL
            } else {
                NOBODY
            }
        }
        Zone::Discard | Zone::LostZone | Zone::Stadium | Zone::PlayArea => ALL,
        // Face down while the game is set up (SSP rulebook p. 8, steps 5 and 6).
        Zone::InPlay { .. } if g.st.in_play_face_down() => only(place.owner),
        Zone::InPlay { .. } => ALL,
        Zone::Looking { from } if kind == EventKind::Look => look_seen_by(&g.st.rules, from, by, place.owner),
        Zone::Looking { from } if public(from) => ALL,
        Zone::Looking { .. } => NOBODY,
    }
}

/// The player an entry's card is looked at by: its cause's, else its own.
fn looker(e: &RecordEntry) -> u8 {
    e.cause.map_or(e.player, |c| c.player)
}

fn entry(kind: EventKind, player: u8, cause: Option<Cause>) -> RecordEntry {
    RecordEntry { kind, player, cause, card: None, from: None, to: None, target: None, other: None, amount: 0, detail: Detail::None, seen_by: ALL }
}

// ---------------------------------------------------------------------------
// Events and routines

/// Open the event `kind` (its moves are recorded as its own until [`close`]); returns the event it is nested in.
fn open(g: &mut Game, kind: EventKind, player: u8, cause: Option<Cause>) -> Option<Ctx> {
    let saved = g.record.ctx;
    g.record.ctx = Some(Ctx { kind, player, cause, target: None, other: None, start: g.record.len, eff: None, held: None, filled: false });
    saved
}

/// The event is done: the places its cards went to are read, an entry held for it that wasn't filled (the event was
/// prevented, or failed) is dropped, and the event it was nested in goes on.
fn close(g: &mut Game, saved: Option<Ctx>) {
    let Some(ctx) = g.record.ctx.take() else { return };
    if let (Some(i), false) = (ctx.held, ctx.filled) {
        remove(g, i);
    }
    let gr: &Game = g;
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        let b = &mut *b;
        let mut k = 0;
        while k < b.pending.len() {
            let i = b.pending[k];
            if i < ctx.start {
                k += 1;
                continue;
            }
            let e = &mut b.entries[i as usize];
            if let Some(c) = e.card {
                let to = gr.locate_card(c);
                e.to = to;
                e.seen_by = or(e.seen_by, sees(gr, c, to, e.kind, looker(e)));
            }
            b.pending.swap_remove(k);
        }
    });
    g.record.ctx = saved;
}

/// A routine being recorded ([`begin`] .. [`end`]): the event it is nested in.
pub(crate) struct Routine(Option<Ctx>);

/// The routine `kind` of `player` starts: the cards that move until [`end`] are recorded as its moves (`None` while the
/// game doesn't record).
#[inline]
pub(crate) fn begin(g: &mut Game, kind: EventKind, player: u8, cause: Option<Cause>) -> Option<Routine> {
    if !g.record.on {
        return None;
    }
    Some(Routine(open(g, kind, player, cause)))
}

/// The routine [`begin`] started is done.
#[inline]
pub(crate) fn end(g: &mut Game, r: Option<Routine>) {
    if let Some(Routine(saved)) = r {
        close(g, saved);
    }
}

/// Run `f` as the routine `kind` of `player` ([`begin`], `f`, [`end`]).
#[inline]
pub(crate) fn scope<T>(g: &mut Game, kind: EventKind, player: u8, cause: Option<Cause>, f: impl FnOnce(&mut Game) -> T) -> T {
    let r = begin(g, kind, player, cause);
    let out = f(g);
    end(g, r);
    out
}

/// The event an effect carries, for the record: its kind (`None` for a step inside another event: the effect of an
/// attack or an Ability, the after-attack effects, the start of a retreat), its player.
fn kind_of(e: &Effect) -> Option<(EventKind, u8)> {
    use Effect::*;
    Some(match *e {
        BeginTurn { p } => (EventKind::BeginTurn, p),
        EndTurn { p } => (EventKind::EndTurn, p),
        BetweenTurns { p, .. } => (EventKind::Checkup, p),
        AfterAttackTriggers { p, .. } => (EventKind::AttackDone, p),
        AfterAttack { .. } | BeforeDoingDamage { .. } | RetreatStart { .. } | Power { .. } | Attack { .. } => return None,
        Retreat { p, .. } => (EventKind::Retreat, p),
        UseAttack { p, .. } => (EventKind::UseAttack, p),
        UseStadium { p, .. } => (EventKind::UseStadium, p),
        UsePower { p, .. } => (EventKind::UseAbility, p),
        KnockOut { p, .. } => (EventKind::KnockOut, p),
        LeavePlay { p, .. } => (EventKind::LeavePlay, p),
        TakePrizes { p, .. } => (EventKind::TakePrizes, p),
        Heal { p, .. } => (EventKind::RemoveCounters, p),
        GainCondition { p, .. } => (EventKind::GainCondition, p),
        RemoveCondition { p, .. } => (EventKind::RemoveCondition, p),
        CoinFlip { p, .. } => (EventKind::CoinFlip, p),
        Evolve { p, .. } => (EventKind::Evolve, p),
        EnterPlay { p, .. } => (EventKind::EnterPlay, p),
        Devolve { p, .. } => (EventKind::Devolve, p),
        Swap { p, .. } => (EventKind::Swap, p),
        ApplyEffect { p, .. } => (EventKind::ApplyEffect, p),
        Discard { p, .. } => (EventKind::Discard, p),
        PutIntoHand { p, .. } => (EventKind::PutIntoHand, p),
        PutIntoDeck { p, .. } => (EventKind::PutIntoDeck, p),
        Draw { p, .. } => (EventKind::Draw, p),
        PlaceCounters { p, .. } => (EventKind::PlaceCounters, p),
        MoveCounters { p, .. } => (EventKind::MoveCounters, p),
        ChangeActive { p, .. } => (EventKind::ChangeActive, p),
        Damage { b, .. } => (EventKind::Damage, b.target.p),
        Attach { p, .. } => (EventKind::Attach, p),
        MoveEnergy { p, .. } => (EventKind::MoveEnergy, p),
        MoveTool { p, .. } => (EventKind::MoveTool, p),
        PlayTrainer { p, .. } => (EventKind::PlayTrainer, p),
    })
}

/// Whether the event has entries of its own (otherwise only its cards' moves).
fn has_entries(kind: EventKind) -> bool {
    !matches!(
        kind,
        EventKind::LeavePlay
            | EventKind::TakePrizes
            | EventKind::Evolve
            | EventKind::EnterPlay
            | EventKind::Devolve
            | EventKind::Swap
            | EventKind::Discard
            | EventKind::PutIntoHand
            | EventKind::PutIntoDeck
            | EventKind::Draw
            | EventKind::Attach
            | EventKind::MoveEnergy
            | EventKind::MoveTool
    )
}

/// `Game::reduce_effect` of a recording game: the effect's event is open while its handlers and its consequences run.
pub(crate) fn effect(g: &mut Game, id: EffId) -> crate::game::R {
    let e = *g.e(id);
    let Some((kind, player)) = kind_of(&e) else { return g.reduce_effect_now(id) };
    let saved = open(g, kind, player, e.cause());
    if has_entries(kind) {
        let i = push(g, entry(kind, player, e.cause()), false);
        if let Some(c) = g.record.ctx.as_mut() {
            c.eff = Some(id);
            c.held = Some(i);
        }
    } else if let Some(c) = g.record.ctx.as_mut() {
        c.eff = Some(id);
    }
    let r = g.reduce_effect_now(id);
    close(g, saved);
    r
}

fn top(g: &Game, s: crate::effects::SlotRef) -> Option<CardId> {
    g.st.slot_pokemon(s.p as usize, s.s)
}

/// The effect `id` is about to have its consequences (its handlers ran, it isn't prevented): its entries are made from
/// the state before them, and the Pokémon its moves are about.
pub(crate) fn before_consequences(g: &mut Game, id: EffId) {
    let Some(ctx) = g.record.ctx else { return };
    if ctx.eff != Some(id) {
        return;
    }
    use Effect::*;
    let e = *g.e(id);
    let (kind, player) = (ctx.kind, ctx.player);
    let base = entry(kind, player, e.cause());
    let mut out: crate::list::SVec<(RecordEntry, bool), 8> = crate::list::SVec::new();
    let (mut target, mut other) = (None, None);
    match e {
        BeginTurn { .. } | EndTurn { .. } | BetweenTurns { .. } => out.push((base, false)),
        AfterAttackTriggers { attack, .. } => out.push((RecordEntry { card: g.last_attack.as_ref().and_then(|l| l.pokemon), detail: Detail::Attack(attack), ..base }, false)),
        Retreat { p, .. } => out.push((RecordEntry { card: g.st.active_pokemon(p as usize), ..base }, false)),
        UseAttack { attack, source, .. } => out.push((RecordEntry { card: top(g, source), detail: Detail::Attack(attack), ..base }, false)),
        UseStadium { stadium, .. } => out.push((RecordEntry { card: Some(stadium), ..base }, false)),
        UsePower { power, card, .. } => out.push((RecordEntry { card: Some(card), detail: Detail::Power(power), ..base }, false)),
        PlayTrainer { card, use_, .. } => out.push((RecordEntry { card: Some(card), detail: Detail::Trainer(use_), ..base }, false)),
        KnockOut { target: t, .. } => out.push((RecordEntry { card: top(g, t), target: top(g, t), ..base }, false)),
        Heal { target: t, damage, .. } => out.push((RecordEntry { card: top(g, t), target: top(g, t), amount: damage, ..base }, false)),
        // A Special Condition the Pokémon already has, the same way, changes nothing: no entry (`condition::gain_changes`).
        GainCondition { target: t, condition, counters, .. } if !crate::engine::condition::gain_changes(g.st.slot(t.p as usize, t.s), condition, counters) => {}
        GainCondition { target: t, condition, .. } | RemoveCondition { target: t, condition, .. } => {
            out.push((RecordEntry { card: top(g, t), target: top(g, t), detail: Detail::Condition(condition), ..base }, false))
        }
        CoinFlip { purpose, heads, .. } => out.push((RecordEntry { detail: Detail::Coin { purpose, heads }, ..base }, false)),
        ApplyEffect { target: t, card, .. } => {
            let on = match t {
                ApplyTarget::Slot(s) => top(g, s),
                ApplyTarget::Player(_) => None,
            };
            out.push((RecordEntry { card: Some(card), target: on, ..base }, false))
        }
        PlaceCounters { target: t, amount, .. } => out.push((RecordEntry { card: top(g, t), target: top(g, t), amount, ..base }, false)),
        Damage { b, amount, .. } => out.push((RecordEntry { card: top(g, b.target), target: top(g, b.target), amount, ..base }, false)),
        MoveCounters { buf, .. } => {
            for m in g.fxp.moves[buf as usize].iter() {
                let to = top(g, m.to);
                let e = RecordEntry { card: to, target: to, other: top(g, m.from), amount: m.placed as i32, detail: Detail::Counters { removed: m.removed as i32, placed: m.placed as i32 }, ..base };
                if out.len() < out.capacity() {
                    out.push((e, false));
                }
            }
        }
        ChangeActive { p, from, to, .. } => {
            // The two Pokémon change spots: each entry's places are its Pokémon's spots before and after.
            let (coming, going) = (g.st.slot_pokemon(p as usize, to), g.st.slot_pokemon(p as usize, from));
            if let Some(c) = coming {
                out.push((RecordEntry { card: Some(c), from: g.locate_card(c), other: going, ..base }, true));
            }
            if let Some(c) = going {
                out.push((RecordEntry { card: Some(c), from: g.locate_card(c), other: coming, ..base }, true));
            }
        }
        Evolve { base: b, .. } => target = Some(b),
        Devolve { target: t, .. } => target = top(g, t),
        Swap { old, .. } => other = Some(old),
        Attach { target: t, .. } => target = top(g, t),
        MoveEnergy { from, to, .. } | MoveTool { from, to, .. } => {
            target = top(g, to);
            other = top(g, from);
        }
        LeavePlay { target: t, pokemon, .. } => target = if pokemon != crate::list::NO_CARD { Some(pokemon) } else { t.and_then(|t| top(g, t)) },
        EnterPlay { .. } | Discard { .. } | PutIntoHand { .. } | PutIntoDeck { .. } | Draw { .. } | TakePrizes { .. } => {}
        AfterAttack { .. } | BeforeDoingDamage { .. } | RetreatStart { .. } | Power { .. } | Attack { .. } => {}
    }
    for (k, (mut x, pending)) in out.iter().copied().enumerate() {
        if pending {
            if let Some(c) = x.card {
                x.seen_by = sees(g, c, x.from, x.kind, looker(&x));
            }
        }
        match (k, ctx.held) {
            (0, Some(i)) => {
                BUF.with(|b| b.borrow_mut().entries[i as usize] = x);
                if pending {
                    BUF.with(|b| b.borrow_mut().pending.push(i));
                }
            }
            (_, Some(i)) => insert(g, i + k as u32, x, pending),
            (_, None) => {
                push(g, x, pending);
            }
        }
    }
    if let Some(c) = g.record.ctx.as_mut() {
        c.target = target;
        c.other = other;
        c.filled = !out.is_empty();
    }
}

/// Card `c` is about to move from `src` to `dst` (`Game::move_card_to` / `move_to`): one entry, `to` read when the
/// event is done. A card leaving a place it went to earlier in the events still open has that entry's `to` read now.
pub(crate) fn moving(g: &mut Game, src: ListRef, c: CardId, dst: ListRef) {
    if src == dst {
        return;
    }
    let present = match src {
        ListRef::Slot(p, s) | ListRef::SlotEnergies(p, s) => {
            let sl = g.st.slot(p as usize, s);
            sl.cards().contains(c) || sl.energies().contains(c) || sl.tools().contains(c)
        }
        _ => g.lst(src).contains(&c),
    };
    if !present {
        return;
    }
    let Some(ctx) = g.record.ctx else {
        assert!(!crate::game::verify_cache(), "card {c} moved from {src:?} to {dst:?} with no event open for the record");
        BUF.with(|b| b.borrow_mut().unattributed += 1);
        return;
    };
    let from = g.locate_card(c);
    let gr: &Game = g;
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        let b = &mut *b;
        let mut k = 0;
        while k < b.pending.len() {
            let i = b.pending[k] as usize;
            if b.entries[i].card == Some(c) {
                let e = &mut b.entries[i];
                e.to = from;
                e.seen_by = or(e.seen_by, sees(gr, c, from, e.kind, looker(e)));
                b.pending.swap_remove(k);
            } else {
                k += 1;
            }
        }
    });
    let e = RecordEntry { card: Some(c), from, target: ctx.target, other: ctx.other, ..entry(ctx.kind, ctx.player, ctx.cause) };
    let e = RecordEntry { seen_by: sees(g, c, from, e.kind, looker(&e)), ..e };
    push(g, e, true);
}

/// The first `n` cards of `src` (and, for a whole Pokémon spot, the cards attached as Energy only) are about to move to
/// `dst` (`Game::move_to`).
pub(crate) fn moving_first(g: &mut Game, src: ListRef, n: usize, dst: ListRef) {
    let mut cards: crate::list::SVec<CardId, 120> = crate::list::SVec::new();
    for &c in g.lst(src).iter().take(n) {
        cards.push(c);
    }
    if let ListRef::Slot(p, s) = src {
        let sl = g.st.slot(p as usize, s);
        if n >= sl.cards().len() {
            for c in sl.energies().iter() {
                if !sl.cards().contains(c) {
                    cards.push(c);
                }
            }
        }
    }
    for &c in cards.iter() {
        moving(g, src, c, dst);
    }
}

/// `cards` are shown to player `to` (the Reveal routine): one entry per card, seen by `to` and by whoever sees it where
/// it is.
pub(crate) fn reveal(g: &mut Game, to: usize, cards: &[CardId]) {
    if !g.record.on {
        return;
    }
    let cause = g.record.ctx.and_then(|c| c.cause);
    for &c in cards {
        let at = g.locate_card(c);
        let e = RecordEntry { card: Some(c), seen_by: or(only(to as u8), sees(g, c, at, EventKind::Reveal, to as u8)), ..entry(EventKind::Reveal, to as u8, cause) };
        push(g, e, false);
    }
}

/// The Pokémon put down face down at setup are turned face up (SSP rulebook p. 8 step 8, p. 18 step 4): a Reveal to both
/// players per card in play.
pub(crate) fn setup_revealed(g: &mut Game) {
    if !g.record.on {
        return;
    }
    for p in 0..2 {
        let pl = &g.st.players[p];
        let mut cards: crate::list::SVec<CardId, 120> = crate::list::SVec::new();
        for s in pl.in_play() {
            for c in pl.slots[s as usize].cards().iter() {
                cards.push(c);
            }
        }
        for &c in cards.iter() {
            let e = RecordEntry { card: Some(c), ..entry(EventKind::Reveal, p as u8, Some(Cause::rule(RuleWhich::Setup, p as u8))) };
            push(g, e, false);
        }
    }
}

/// The setup check of player `p`'s hand for a Basic Pokémon (SSP rulebook p. 8 step 4).
pub(crate) fn basic_check(g: &mut Game, p: usize, found: bool) {
    if !g.record.on {
        return;
    }
    let e = RecordEntry { detail: Detail::BasicCheck { found }, ..entry(EventKind::Mulligan, p as u8, Some(Cause::rule(RuleWhich::Setup, p as u8))) };
    push(g, e, false);
}

/// A damage (`EventKind::Damage`), damage counters (`PlaceCounters`) or a heal (`RemoveCounters`) to the Pokémon in
/// `target` comes out 0 or is refused: no event happens (RULES.md "Events batch 10, E14"); the record shows it as an entry
/// of that kind with amount 0, made like the entry of one that happens (the Pokémon as `card` and `target`, public).
#[inline]
pub(crate) fn comes_out_zero(g: &mut Game, kind: EventKind, target: crate::effects::SlotRef, cause: Cause) {
    if !g.record.on {
        return;
    }
    let Some(c) = top(g, target) else { return };
    push(g, RecordEntry { card: Some(c), target: Some(c), ..entry(kind, target.p, Some(cause)) }, false);
}

/// Player `p`'s deck is shuffled (a ShuffleDeck prompt is answered).
pub(crate) fn shuffle(g: &mut Game, p: usize) {
    if !g.record.on {
        return;
    }
    let cause = g.record.ctx.and_then(|c| c.cause);
    push(g, entry(EventKind::Shuffle, p as u8, cause), false);
}

/// The game ends (`winner`, `reason`; no reason for a tie that starts a Tiebreaker game).
pub(crate) fn game_end(g: &mut Game, winner: Winner, reason: Option<EndReason>) {
    if !g.record.on {
        return;
    }
    push(g, RecordEntry { detail: Detail::End { winner, reason }, ..entry(EventKind::GameEnd, NO_PLAYER, None) }, false);
}

#[cfg(test)]
mod tests;
