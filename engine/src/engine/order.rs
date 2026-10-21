//! The order of simultaneous triggers (events batch 9 C7a; batch 8 C11c's prompt, generalized): one owner for every window
//! in which several effects activate at the same time and a player decides in which order they apply.
//!
//! A window collects its pending effects ([`Pending`], `Game::pending`), then they are applied one at a time. The order is
//! the order recorded (the propagation order of the declaring cards) unless two or more distinct effects apply: then the
//! window's chooser picks the next one with a Select prompt over the effects' cards, one at a time, each effect's own
//! choices made before the next ([`step`], [`chosen`]). Identical effects (the same card, the same trigger, the same
//! player: two Freezing Shrouds of one player) are one item: the order between them can't change the game (C10's rule). An
//! effect applies when its trigger fires and its program can change the game ([`crate::spec::run::applies`]); the "this
//! turn" marker bookkeeping isn't an effect.
//!
//! The windows ([`Window`]) and their choosers (the APR line each cites):
//! - E-03 the effects activated by a Pokémon being damaged by an attack, after the attack's damage and effects: the player
//!   of the damaged Pokémon;
//! - E-04 the effects activated by Knock Outs, step 2 of the Knock Out process: the player whose turn is being played;
//! - E-21 the effects at the end of a turn: the player whose turn is ending;
//! - F, Pokémon Checkup step 2: the player whose turn would be next, who also decides for each effect whether it comes
//!   before or after checking all Special Conditions (a pseudo item of the same prompt);
//! - the effects activated by Pokémon an effect puts into play ("whenever a player puts a Pokémon onto their Bench"), after
//!   that effect is done (JP FAQ Risky Ruins x Call for Family x Solrock / Lunatone with New Moon: the Pokémon are all on the
//!   Bench first): the player whose turn it is (compendium 1853, 1923: simultaneous effects, the player chooses; as for the
//!   effects of attaching Energy, APR E-07: "the player whose turn it is currently decides").
//!
//! E-03 and E-04 are driven by their callers' loops (the attack's step 7, the state check), which call [`step`] until
//! [`pending`] is false and wait for the prompts a step opened; E-21, F and the put-into-play window are chained ([`next`]:
//! a step, the effect's prompts answered, the next step; [`crate::engine::phase::window_done`] when nothing is left). The
//! put-into-play window opens when the effect's program ends ([`after_effect`]).

use crate::effects::{AtkBase, EffId, Effect};
use crate::game::{Cont, Game, OnComplete, R};
use crate::list::*;
use crate::prompts::{PromptKind, SelectValues};
use crate::spec::run::{self, Timed};
use crate::types::GamePhase;

/// One effect waiting in a window.
#[derive(Clone, Copy, Debug)]
pub enum Item {
    /// Trigger `i` of card `c` that fires on the event `e` (a Knock Out, the end of a turn, a Checkup).
    Card(Timed),
    /// Trigger `i` of card `c` over the Damage `b.target` took from an attack (APR E-03): the card's "is damaged by an
    /// attack" effect, recorded when the damage is done.
    Damaged(DamagedTrigger),
    /// The lasting trigger of the damaged Pokémon "if this Pokémon is damaged by an attack, put N damage counters on the
    /// Attacking Pokémon" (`engine::damage::retaliate`), recorded when the damage is done.
    Retaliation(crate::engine::damage::Retaliation),
    /// The lasting trigger "when this Pokémon is Knocked Out" armed on the Pokémon of the KnockOut `e`
    /// (`knockout::lasting_ko_trigger`).
    LastingKo { e: EffId },
    /// Trigger `i` of card `c` over the EnterPlay of a Pokémon an effect put into play, recorded when it entered and applied
    /// after the effect ([`Window::PutIntoPlay`]).
    Entered(EnteredTrigger),
    /// Trigger `i` of card `c` over the event `e`, held until the Trainer card whose effect made the event is done
    /// (`TableRules::evolving_trainer_discarded_before_evolve_triggers`); the event is retained until it fires.
    Held(Timed),
}

/// A trigger over the EnterPlay of a Pokémon an effect put into play: the card and its trigger, the program's player, and the
/// event as it was (the program reads it again when it applies: the EnterPlay event is made once more for it,
/// `run::fire_entered`).
#[derive(Clone, Copy, Debug)]
pub struct EnteredTrigger {
    pub c: CardId,
    pub i: u8,
    pub p: u8,
    pub card: CardId,
    pub target: crate::effects::SlotRef,
    pub from: crate::state::ListRef,
    pub source: crate::spec::event::RulesZone,
    pub cause: crate::cause::Cause,
}

/// A trigger over the Damage an attack did (E-03): the card and its trigger, and the Damage as it was when it was done (the
/// program reads it again when it applies: the Damage event is made once more for it, `run::fire_damaged`).
#[derive(Clone, Copy, Debug)]
pub struct DamagedTrigger {
    pub c: CardId,
    pub i: u8,
    pub b: AtkBase,
    pub amount: i32,
}

/// Which window a pending effect belongs to (the windows of nested attacks are kept apart by the attack's effect).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    Damaged(EffId),
    KnockOut,
    EndTurn,
    Checkup,
    /// The Pokémon the program of the card put into play.
    PutIntoPlay(CardId),
    /// The triggers held until a Trainer card was done (a table rule).
    TrainerDone,
}

/// Whether the window asks for the order of its effects, and for a window that does, whether the effect applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Just recorded (a step 7 record: the window is decided at its first step, when all the records are in).
    Recorded,
    /// Applied in the order recorded.
    Plain,
    /// Two or more distinct effects apply: the chooser picks the order of those that apply; those that don't go first, in
    /// the order recorded.
    Ordered { applies: bool },
}

#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub tag: Tag,
    pub item: Item,
    pub mode: Mode,
}

/// A simultaneous-trigger window, with what follows it for the windows that chain.
#[derive(Clone, Copy, Debug)]
pub enum Window {
    /// APR E-03, the attack `atk`'s step 7.
    Damaged { atk: EffId },
    /// APR E-04, step 2 of one state check's Knock Outs.
    KnockOut,
    /// APR E-21, the end of player `p`'s turn (the EndTurn event `e`; the end of the turn follows).
    EndTurn { p: u8, e: EffId },
    /// APR F step 2: Pokémon Checkup's effects. `sc`: the Special Conditions are still to be checked (an item of the order
    /// when an Active Pokémon has one); `several`: two or more distinct effects applied at the start (the before / after
    /// choice exists only then); the rest of Pokémon Checkup follows (the Special Conditions' amounts per player, the
    /// Checkup events, the continuation).
    Checkup { oc: OnComplete, poison: [i32; 2], burn: [i32; 2], e: [EffId; 2], several: bool, sc: bool },
    /// The effects activated by the Pokémon the program of card `by` put into play, once that program is done.
    PutIntoPlay { by: CardId },
    /// The triggers held while player `p`'s Trainer card was played, once the card is done and discarded (a table rule:
    /// `TableRules::evolving_trainer_discarded_before_evolve_triggers`).
    TrainerDone { p: u8 },
}

impl Window {
    pub fn tag(&self) -> Tag {
        match *self {
            Window::Damaged { atk } => Tag::Damaged(atk),
            Window::KnockOut => Tag::KnockOut,
            Window::EndTurn { .. } => Tag::EndTurn,
            Window::Checkup { .. } => Tag::Checkup,
            Window::PutIntoPlay { by } => Tag::PutIntoPlay(by),
            Window::TrainerDone { .. } => Tag::TrainerDone,
        }
    }

    /// Who picks the order of the effects: the player of the damaged Pokémon (APR E-03: "the player of the Pokémon taking
    /// the damage gets to choose in which order to apply those effects"); the player whose turn is currently being played
    /// (E-04: "the player whose turn is currently being played gets to decide in which order to apply those effects"; a
    /// Knock Out at Pokémon Checkup, in neither player's turn, is the next player's as everything at Checkup is: F step 2,
    /// D step 5); the player whose turn is ending (E-21); the player whose turn would be next (F step 2, E-08); the player
    /// whose turn it is for the effects of Pokémon put into play (compendium 1853, 1923).
    pub fn chooser(&self, g: &Game) -> usize {
        match *self {
            Window::Damaged { atk } => match *g.e(atk) {
                Effect::Attack { opp, .. } => opp as usize,
                _ => 1 - g.st.active_player as usize,
            },
            Window::KnockOut if g.st.phase == GamePhase::BetweenTurns => 1 - g.st.active_player as usize,
            Window::KnockOut => g.st.active_player as usize,
            Window::EndTurn { p, .. } => p as usize,
            Window::Checkup { .. } => 1 - g.st.active_player as usize,
            Window::PutIntoPlay { .. } => g.st.active_player as usize,
            // The player who played the Trainer (it is their turn).
            Window::TrainerDone { p } => p as usize,
        }
    }

    /// The windows a step does not continue by itself: the caller's loop waits for the step's prompts and steps again.
    fn chains(&self) -> bool {
        matches!(self, Window::EndTurn { .. } | Window::Checkup { .. } | Window::PutIntoPlay { .. } | Window::TrainerDone { .. })
    }

    /// The Special Conditions check is an item of the order now (Checkup, an Active Pokémon has a Special Condition).
    fn sc_item(&self, g: &Game) -> bool {
        matches!(self, Window::Checkup { sc: true, .. }) && crate::engine::phase::special_conditions_item(g)
    }
}

/// What a step of a window did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// The effects are all applied (and the Special Conditions checked).
    Done,
    /// The order prompt is open: its answer applies the chosen effect ([`chosen`]).
    Asked,
    /// One effect (or the Special Conditions check) was applied; its prompts may be open.
    Fired,
}

// ---------------------------------------------------------------------------
// The effects

fn def_of(g: &Game, c: CardId) -> u16 {
    g.st.cards[c as usize].def
}

/// Two effects are the same item of the order prompt when they are the same effect of the same kind of card for the same
/// player (the order between them can't change the game; batch 8 C12b, review 8b Must 4).
fn same_item(g: &Game, a: &Item, b: &Item) -> bool {
    match (a, b) {
        (Item::Card(a), Item::Card(b)) | (Item::Held(a), Item::Held(b)) => a.i == b.i && a.p == b.p && def_of(g, a.c) == def_of(g, b.c),
        (Item::Damaged(a), Item::Damaged(b)) => a.i == b.i && def_of(g, a.c) == def_of(g, b.c) && a.b.target.p == b.b.target.p,
        (Item::Retaliation(a), Item::Retaliation(b)) => def_of(g, a.lasting.source) == def_of(g, b.lasting.source) && a.target.p == b.target.p,
        (Item::LastingKo { e: a }, Item::LastingKo { e: b }) => matches!((*g.e(*a), *g.e(*b)), (Effect::KnockOut { p: x, .. }, Effect::KnockOut { p: y, .. }) if x == y),
        (Item::Entered(a), Item::Entered(b)) => a.i == b.i && a.p == b.p && def_of(g, a.c) == def_of(g, b.c),
        _ => false,
    }
}

/// The card whose effect `item` is (the card the prompt shows for it).
fn source_card(g: &Game, item: &Item) -> Option<CardId> {
    match *item {
        Item::Card(t) | Item::Held(t) => Some(t.c),
        Item::Damaged(t) => Some(t.c),
        Item::Retaliation(t) => Some(t.lasting.source),
        Item::LastingKo { e } => crate::engine::knockout::lasting_ko_source(g, e),
        Item::Entered(t) => Some(t.c),
    }
}

/// Apply one effect (it re-checks what may have changed: still attached, not blocked, the attacker still in play).
fn fire(g: &mut Game, item: Item) -> R {
    match item {
        Item::Card(t) => run::fire_timed(g, t),
        Item::Held(t) => run::fire_held(g, t),
        Item::Damaged(t) => run::fire_damaged(g, &t),
        Item::Retaliation(t) => crate::engine::damage::retaliate(g, &t),
        Item::LastingKo { e } => crate::engine::knockout::lasting_ko_trigger(g, e),
        Item::Entered(t) => run::fire_entered(g, &t),
    }
}

/// The distinct effects among `idx` (indices into `Game::pending`): the first of each, in the order recorded.
fn heads(g: &Game, idx: &[usize]) -> SVec<u8, 32> {
    let mut out: SVec<u8, 32> = SVec::new();
    for (k, &i) in idx.iter().enumerate() {
        let item = &g.pending.as_slice()[i].item;
        if !idx[..k].iter().any(|&j| same_item(g, &g.pending.as_slice()[j].item, item)) {
            out.push(k as u8);
        }
    }
    out
}

/// Two or more distinct effects apply among `items` (each with whether it applies): the window asks for the order (the order
/// between identical ones can't change the game; APR E-03, E-04, E-21: "several such effects").
pub fn asks(g: &Game, items: &[(Item, bool)]) -> bool {
    let mut seen: SVec<u8, 32> = SVec::new();
    for (k, (a, x)) in items.iter().enumerate() {
        if *x && !items[..k].iter().any(|(b, y)| *y && same_item(g, a, b)) {
            seen.push(k as u8);
        }
    }
    seen.len() >= 2
}

// ---------------------------------------------------------------------------
// The window's list

fn push_pending(g: &mut Game, p: Pending) {
    assert!(g.pending.len() < g.pending.capacity(), "more than {} effects waiting in simultaneous-trigger windows", g.pending.capacity());
    g.pending.push(p);
}

/// Record an effect of the window `tag` as it happens (a step 7 record while the attack's damage is done).
pub fn record(g: &mut Game, tag: Tag, item: Item) {
    push_pending(g, Pending { tag, item, mode: Mode::Recorded });
}

/// Collect the effects of a window, each with whether it applies, in the order recorded: the chooser orders them when two or
/// more distinct ones apply ([`asks`]), otherwise they are applied in the order recorded.
pub fn collect(g: &mut Game, tag: Tag, items: &[(Item, bool)]) {
    let ordered = asks(g, items);
    for &(item, x) in items {
        push_pending(g, Pending { tag, item, mode: if ordered { Mode::Ordered { applies: x } } else { Mode::Plain } });
    }
}

/// Open a chained window whose effects ask for an order ([`asks`]): collect them and take the first step.
pub fn open(g: &mut Game, w: Window, items: &[(Item, bool)]) -> R {
    collect(g, w.tag(), items);
    next(g, w)
}

/// Effects of the window `tag` are waiting.
pub fn pending(g: &Game, tag: Tag) -> bool {
    g.pending.as_slice().iter().any(|p| p.tag == tag)
}

/// Drop the effects of the window `tag` (the attack is gone with its window).
pub fn drop_window(g: &mut Game, tag: Tag) {
    g.pending.retain(|p| p.tag != tag);
}

fn indices(g: &Game, tag: Tag) -> SVec<usize, 32> {
    let mut idx: SVec<usize, 32> = SVec::new();
    for (i, p) in g.pending.as_slice().iter().enumerate() {
        if p.tag == tag {
            idx.push(i);
        }
    }
    idx
}

/// A window of records (step 7's, the put-into-play window's) is decided at its first step: ordered when two or more distinct
/// effects apply now.
fn decide(g: &mut Game, idx: &[usize]) {
    let mut items: SVec<(Item, bool), 32> = SVec::new();
    for &i in idx {
        let item = g.pending.as_slice()[i].item;
        // Only a record is undecided; a single effect is never asked about.
        let x = match item {
            Item::Damaged(t) if idx.len() >= 2 => run::damaged_applies(g, &t),
            Item::Retaliation(t) if idx.len() >= 2 => crate::engine::damage::retaliation_applies(g, &t),
            Item::Entered(t) if idx.len() >= 2 => run::entered_applies(g, &t),
            _ => false,
        };
        items.push((item, x));
    }
    let ordered = asks(g, items.as_slice());
    for (k, &i) in idx.iter().enumerate() {
        g.pending.as_mut_slice()[i].mode = if ordered { Mode::Ordered { applies: items.as_slice()[k].1 } } else { Mode::Plain };
    }
}

// ---------------------------------------------------------------------------
// The routine

/// One step of the window: the effects that don't apply first (when the window asks), then the order prompt while two or
/// more distinct effects remain (and the Special Conditions check at Checkup is one of them), else the next effect in the
/// order recorded; at Checkup with no choice the Special Conditions come first (APR F step 1).
pub fn step(g: &mut Game, w: &mut Window) -> R<Step> {
    let tag = w.tag();
    let idx = indices(g, tag);
    if idx.as_slice().iter().any(|&i| g.pending.as_slice()[i].mode == Mode::Recorded) {
        decide(g, idx.as_slice());
    }
    let ordered = idx.as_slice().first().map_or(false, |&i| matches!(g.pending.as_slice()[i].mode, Mode::Ordered { .. }));
    if ordered {
        if let Some(&i) = idx.as_slice().iter().find(|&&i| g.pending.as_slice()[i].mode == (Mode::Ordered { applies: false })) {
            let p = g.pending.remove_at(i);
            fire(g, p.item)?;
            return Ok(Step::Fired);
        }
    }
    let hs = heads(g, idx.as_slice());
    let items = hs.len() + w.sc_item(g) as usize;
    let asks = match *w {
        Window::Checkup { several, .. } => several,
        _ => ordered,
    };
    if items >= 2 && asks {
        let mut names: SVec<(&'static str, Option<CardId>), 8> = SVec::new();
        for &k in hs.iter() {
            assert!(names.len() < names.capacity(), "more than 8 distinct effects to order");
            let c = source_card(g, &g.pending.as_slice()[idx.as_slice()[k as usize]].item);
            names.push((g.st.cdef(c.unwrap_or(0)).full_name, c));
        }
        if w.sc_item(g) {
            assert!(names.len() < names.capacity(), "more than 8 distinct effects to order");
            names.push(("Special Conditions", None));
        }
        g.prompt(
            w.chooser(g),
            crate::purpose::PromptPurpose::OrderEffects,
            PromptKind::Select { values: SelectValues::Effects(names), allow_cancel: false, default_value: 0 },
            crate::prompts::Provenance::NONE,
            Cont::OrderEffects { w: *w },
        );
        return Ok(Step::Asked);
    }
    if let Window::Checkup { sc: true, .. } = *w {
        // No choice: the rulebook's order, the Special Conditions check first (APR F step 1), then the effects (step 2).
        crate::engine::phase::checkup_conditions(g, w)?;
        return Ok(Step::Fired);
    }
    match idx.as_slice().first() {
        Some(&i) => {
            let p = g.pending.remove_at(i);
            fire(g, p.item)?;
            Ok(Step::Fired)
        }
        None => Ok(Step::Done),
    }
}

/// The choice of the order prompt: that effect applies now (its own prompts are answered before the next step).
pub fn chosen(g: &mut Game, mut w: Window, choice: i32) -> R {
    let idx = indices(g, w.tag());
    let hs = heads(g, idx.as_slice());
    let k = choice.max(0) as usize;
    if k < hs.len() {
        // The item's first effect (identical ones follow as the same item).
        let p = g.pending.remove_at(idx.as_slice()[hs.as_slice()[k] as usize]);
        fire(g, p.item)?;
    } else if k == hs.len() && w.sc_item(g) {
        crate::engine::phase::checkup_conditions(g, &mut w)?;
    } else {
        // An answer outside the offered effects.
        crate::bail!("INVALID_PROMPT_RESULT");
    }
    if w.chains() {
        return next(g, w);
    }
    Ok(())
}

/// The program of card `by` is done: the effects activated by the Pokémon it put into play apply now (the put-into-play
/// window), in the order recorded or, with two or more distinct effects that apply, in the order the turn player picks.
/// A plain read when no window waits.
pub fn after_effect(g: &mut Game, by: CardId) -> R {
    if g.pending.is_empty() || !pending(g, Tag::PutIntoPlay(by)) {
        return Ok(());
    }
    next(g, Window::PutIntoPlay { by })
}

/// A chained window after an effect: once its prompts are answered, the next step, or what follows the last.
pub fn next(g: &mut Game, mut w: Window) -> R {
    if g.has_prompts() {
        g.wait_prompt(Cont::OrderNext { w });
        return Ok(());
    }
    match step(g, &mut w)? {
        Step::Done => crate::engine::phase::window_done(g, w),
        Step::Asked => Ok(()),
        Step::Fired => next(g, w),
    }
}

#[cfg(test)]
mod tests {
    //! The choosers of the windows (APR E-04, E-21, F step 2), and that two effects of one kind of card for one player are one
    //! item of the order prompt. (E-03's chooser, the damaged Pokémon's player, is read from the attack and is asserted by the
    //! scenarios `handheld-fan-twm-150-and-ready-to-ram-damaged-player-orders-*`.)
    use super::*;

    fn game() -> Game {
        let mut deck: Vec<u16> = Vec::new();
        for _ in 0..4 {
            deck.push(crate::carddb::def_by_full_name("Maractus JTG 8").unwrap());
        }
        while deck.len() < 60 {
            deck.push(crate::carddb::def_by_full_name("Grass Energy MEE 1").unwrap());
        }
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        g
    }

    #[test]
    fn the_chooser_of_each_window_is_the_players_the_apr_names() {
        let mut g = game();
        let turn = g.st.active_player as usize;
        // E-04: the player whose turn is currently being played; at Checkup, in neither player's turn, the player whose
        // turn would be next.
        g.st.phase = GamePhase::PlayerTurn;
        assert_eq!(Window::KnockOut.chooser(&g), turn);
        g.st.phase = GamePhase::Attack;
        assert_eq!(Window::KnockOut.chooser(&g), turn);
        g.st.phase = GamePhase::BetweenTurns;
        assert_eq!(Window::KnockOut.chooser(&g), 1 - turn);
        // E-21: the player whose turn is ending. F step 2: the player whose turn would be next.
        assert_eq!(Window::EndTurn { p: turn as u8, e: 0 }.chooser(&g), turn);
        let checkup = Window::Checkup { oc: OnComplete::None, poison: [10, 10], burn: [20, 20], e: [0, 0], several: true, sc: true };
        assert_eq!(checkup.chooser(&g), 1 - turn);
    }

    #[test]
    fn identical_effects_of_one_player_are_one_item() {
        let g = game();
        let maractus: Vec<CardId> = (0..120u8).filter(|c| g.st.cdef(*c).full_name == "Maractus JTG 8").collect();
        assert!(maractus.len() >= 2);
        let t = |c: CardId, i: u8, p: u8| Item::Card(Timed { c, i, p, slot: 0, e: 0 });
        assert!(same_item(&g, &t(maractus[0], 0, 0), &t(maractus[1], 0, 0)), "two copies of one card, the same trigger and player");
        assert!(!same_item(&g, &t(maractus[0], 0, 0), &t(maractus[1], 0, 1)), "another player's effect is another item");
        assert!(!same_item(&g, &t(maractus[0], 0, 0), &t(maractus[1], 1, 0)), "another trigger is another item");
        assert!(asks(&g, &[(t(maractus[0], 0, 0), true), (t(maractus[1], 0, 1), true)]));
        assert!(!asks(&g, &[(t(maractus[0], 0, 0), true), (t(maractus[1], 0, 0), true)]));
        assert!(!asks(&g, &[(t(maractus[0], 0, 0), true), (t(maractus[1], 0, 1), false)]), "an effect that doesn't apply isn't asked about");
    }

    /// A board with Risky Ruins in play and Duskull in the turn player's deck (Basic [P] Pokémon, 60 HP).
    fn ruins_game() -> (Game, usize, Vec<CardId>) {
        let sc = serde_json::json!({"me": {"reset": true, "active": "Duskull PRE 35", "stadium": "Risky Ruins MEG 127"}, "opp": {"reset": true, "active": "Duskull PRE 35"}});
        // At the turn player's first turn decision, no question open.
        let g = crate::place::tests::game_of(&[("Duskull PRE 35", 4), ("Risky Ruins MEG 127", 4), ("Buddy-Buddy Poffin TEF 144", 4)], sc);
        assert!(!g.has_prompts());
        let me = g.st.active_player as usize;
        let duskull = crate::carddb::def_by_full_name("Duskull PRE 35").unwrap();
        let active = g.st.active_pokemon(me);
        let others: Vec<CardId> = (0..g.st.cards.len() as CardId).filter(|c| g.st.cards[*c as usize].def == duskull && g.st.owner(*c) == me && Some(*c) != active).collect();
        (g, me, others)
    }

    fn bench_puts(g: &Game, me: usize, cards: &[CardId]) -> Vec<(CardId, crate::effects::SlotRef)> {
        let open = crate::prefabs::empty_bench_slots(g, me);
        cards.iter().zip(open.iter()).map(|(c, s)| (*c, crate::effects::SlotRef::new(me, *s))).collect()
    }

    /// One effect puts two Pokémon onto the Bench with Risky Ruins in play: the damage counters wait until the effect is done
    /// and are then placed on each (JP FAQ Risky Ruins MEG 127: Call for Family's two Basic Pokémon get counters each; with
    /// Solrock and a Lunatone with New Moon put together, New Moon works: the Pokémon are all on the Bench first). A single
    /// Pokémon put by an effect waits the same way (one rule).
    #[test]
    fn the_triggers_of_pokemon_an_effect_puts_onto_the_bench_wait_for_the_effect() {
        for n in [2usize, 1] {
            let (mut g, me, deck) = ruins_game();
            let by = (0..g.st.cards.len() as CardId).find(|c| g.st.cdef(*c).full_name == "Buddy-Buddy Poffin TEF 144" && g.st.owner(*c) == me).unwrap();
            let ps = bench_puts(&g, me, &deck[..n]);
            let cause = crate::cause::Cause::new(crate::cause::CauseKind::Trainer, Some(by), me as u8);
            crate::engine::enter::put_into_play(&mut g, by, &ps, cause).unwrap();
            for (c, t) in &ps {
                assert_eq!(g.st.slot_pokemon(me, t.s), Some(*c), "on the Bench");
                assert_eq!(g.st.slot(me, t.s).damage, 0, "no counters while the effect isn't done");
            }
            assert!(pending(&g, Tag::PutIntoPlay(by)));
            after_effect(&mut g, by).unwrap();
            assert!(!g.has_prompts(), "identical effects of one card for one player aren't ordered");
            for (_, t) in &ps {
                assert_eq!(g.st.slot(me, t.s).damage, 20, "2 damage counters on each once the effect is done");
            }
            assert!(g.pending.is_empty());
        }
    }

    /// Buddy-Buddy Poffin with Risky Ruins in play: the effect is done after "Then, shuffle your deck" (no rule puts the
    /// effects between the Bench puts and the shuffle), so the counters go on after the shuffle, on both Pokémon.
    #[test]
    fn the_counters_of_a_search_onto_the_bench_go_on_after_its_shuffle() {
        let sc = serde_json::json!({"me": {"reset": true, "active": "Duskull PRE 35", "stadium": "Risky Ruins MEG 127", "hand": ["Buddy-Buddy Poffin TEF 144"]}, "opp": {"reset": true, "active": "Duskull PRE 35"}});
        let mut g = crate::place::tests::game_of(&[("Duskull PRE 35", 4), ("Risky Ruins MEG 127", 4), ("Buddy-Buddy Poffin TEF 144", 4)], sc);
        let me = g.st.active_player as usize;
        let k = g.st.players[me].hand().iter().position(|c| g.st.cdef(c).name == "Buddy-Buddy Poffin").unwrap();
        let play = crate::options::legal_actions(&g).into_iter().map(|o| o.action).find(|a| matches!(a, crate::game::Action::PlayCard { hand_index, .. } if *hand_index as usize == k)).unwrap();
        g.act(play).unwrap();
        let sel = g.select_with(false).unwrap().unwrap();
        let mut picks = Vec::new();
        while picks.len() < 2 {
            let (mask, _) = g.pick_mask(&sel, &picks);
            picks.push((0..mask.len()).find(|j| mask[*j] && !picks.contains(j)).expect("two Duskull to pick"));
        }
        g.answer(&sel, &picks).unwrap();
        let bench: Vec<u8> = g.st.players[me].bench.iter().copied().filter(|s| g.st.slot_pokemon(me, *s).is_some()).collect();
        assert_eq!(bench.len(), 2, "both on the Bench");
        let crate::game::Pending::Chance(i) = g.pending() else { panic!("the shuffle waits") };
        assert!(matches!(g.prompts.as_slice()[i].kind, PromptKind::ShuffleDeck));
        assert!(bench.iter().all(|s| g.st.slot(me, *s).damage == 0), "no counters before the shuffle");
        g.settle().unwrap();
        assert!(bench.iter().all(|s| g.st.slot(me, *s).damage == 20), "2 damage counters on each after the effect");
    }

    /// Two distinct effects waiting in the put-into-play window: the turn player picks the order (compendium 1853, 1923). No
    /// pool card has a second "put onto the Bench" trigger an effect's put reaches (Risky Ruins is the only one; the others
    /// say "when you play this Pokémon from your hand"), so the second item is made distinct by hand (another player's).
    #[test]
    fn two_distinct_effects_of_pokemon_put_into_play_are_ordered_by_the_turn_player() {
        let (mut g, me, deck) = ruins_game();
        let by = deck[2];
        let ruins = g.st.stadium_card().unwrap();
        let ps = bench_puts(&g, me, &deck[..2]);
        let cause = crate::cause::Cause::new(crate::cause::CauseKind::Attack, Some(by), me as u8);
        crate::engine::enter::put_into_play(&mut g, by, &ps, cause).unwrap();
        assert_eq!(g.pending.len(), 2);
        assert!(g.pending.as_slice().iter().all(|p| matches!(p.item, Item::Entered(e) if e.c == ruins)));
        if let Item::Entered(e) = &mut g.pending.as_mut_slice()[1].item {
            e.p = 1 - e.p;
        }
        after_effect(&mut g, by).unwrap();
        let last = g.prompts.len() - 1;
        let sel = g.prompt_select(last);
        assert_eq!(sel.context, crate::interface::SelectContext::SkillOrder);
        assert_eq!(g.prompts.as_slice()[last].player as usize, me, "the turn player orders them");
        // The second first: each effect applies, the window ends.
        g.resolve(last, crate::prompts::Res::Int(1)).unwrap();
        for (_, t) in &ps {
            assert_eq!(g.st.slot(me, t.s).damage, 20);
        }
        assert!(g.pending.is_empty() && !g.has_prompts());
    }

    /// The order question's options name the card whose effect each is (`Opt::source`); nothing else of them is set.
    #[test]
    fn the_order_options_name_their_source_card() {
        let mut g = game();
        let sc = serde_json::json!({"me": {"reset": true, "active": "Maractus JTG 8"}, "opp": {"reset": true, "active": "Maractus JTG 8"}});
        crate::scenario::apply(&mut g, &sc).unwrap();
        let maractus: Vec<CardId> = (0..120u8).filter(|c| g.st.cdef(*c).full_name == "Maractus JTG 8").collect();
        let t = |c: CardId, p: u8| Item::Card(Timed { c, i: 0, p, slot: 0, e: 0 });
        let (a, b) = (maractus[0], maractus[1]);
        collect(&mut g, Tag::EndTurn, &[(t(a, 0), true), (t(b, 1), true)]);
        let mut w = Window::EndTurn { p: 0, e: 0 };
        assert!(matches!(step(&mut g, &mut w), Ok(Step::Asked)));
        // The question just asked is the last prompt of the queue (the setup's own questions come before it).
        let sel = g.prompt_select(g.prompts.len() - 1);
        assert_eq!(sel.context, crate::interface::SelectContext::SkillOrder);
        let sources: Vec<Option<u8>> = sel.options.iter().map(|o| o.source).collect();
        assert_eq!(sources, vec![Some(a), Some(b)]);
        assert!(sel.options.iter().all(|o| o.card_id.is_none() && o.serial.is_none()));
    }
}
