//! The damage events of events batch 6 (docs/design/events-design.md, sections 4 and 4.1): PlaceCounters (and, as they
//! land, MoveCounters and Damage).
//!
//! Every path that puts damage counters on a Pokémon calls its routine here, in the `engine::enter` / `engine::attach` /
//! `engine::condition` pattern: the routine checks the event (a Pokémon is there; the locks, `derived::event_locked`;
//! the preventions, `derived::event_prevented`), produces it (an `Effect` dispatched to the cards), applies its
//! consequences in [`reducer`], and the triggers over it run after. A refused event doesn't happen (`Ok(false)`) and
//! the op it belongs to continues (id2264, id2425: Dusknoir's Cursed Blast still Knocks Out Dusknoir when Battle Cage or
//! Hide 'n' Sneak stops the counters).
//!
//! - [`place`]: "put N damage counters on ..." whatever causes it: an attack's effect, an Ability, a Trainer, a
//!   Stadium (Risky Ruins), a Tool (Punk Helmet), an Energy (Spiky Energy), a Special Condition's rule (Poison and
//!   Burn at the Checkup, Confusion). Damage counters are not damage: no Weakness or Resistance, no "damaged" triggers,
//!   no damage-taken record (APR C-07). The Knock Out waits for the state check after the whole action (id66; all
//!   counters are placed first, id1547).
//!
//! - [`move_counters`]: "move N damage counters from ... to ..." (events design 4.1; APR C-08): one event per action,
//!   carrying its pairs. A lock forbids the whole action (Patrat's Watchful Eye: the counters stay, id2350); per pair,
//!   a protected source keeps its counters (nothing moves, id390, id2150, id2192), a protected destination makes the
//!   counters removed from the source vanish (id2257, id79, id393, id1876; JP FAQ Battle Cage x3). Moving counters
//!   isn't healing and isn't damage (id1995, id2254, id554; id251, id1650). The Knock Outs wait for the state check
//!   after the whole action (id66; a one-at-a-time Ability's use is its own action, id67, id2152).
//!
//! The Cause is the frame's (`spec::run::Frame::cause`) or the rule's; ops never pick one.

use crate::cause::Cause;
use crate::effects::{CounterMove, EffId, Effect, SlotRef};
use crate::list::SVec;
use crate::game::{Game, R};
use crate::spec::event::*;

// ---------------------------------------------------------------------------
// Event views

/// The PlaceCounters event: `amount` HP of damage counters on the Pokémon in `target`.
pub fn counters_view(g: &Game, target: SlotRef, amount: i32, cause: Cause) -> EventView {
    EventView { card: g.st.slot_pokemon(target.p as usize, target.s), slot: Some(target), amount, ..EventView::new(EventKind::PlaceCounters, cause, target.p, crate::spec::event::whose_turn(g)) }
}

/// The MoveCounters event as one end of a move sees it: `amount` HP of counters leaving (`From`) or arriving on (`To`)
/// the Pokémon in `slot` (events design 4.1: a protected source keeps its counters, a protected destination makes them
/// vanish).
pub fn move_view(g: &Game, slot: SlotRef, end: MoveEnd, amount: i32, cause: Cause) -> EventView {
    EventView { card: g.st.slot_pokemon(slot.p as usize, slot.s), slot: Some(slot), amount, end: Some(end), ..EventView::new(EventKind::MoveCounters, cause, slot.p, crate::spec::event::whose_turn(g)) }
}

// ---------------------------------------------------------------------------
// The routines

/// PlaceCounters: `hp` HP of damage counters on the Pokémon in `target`. It doesn't happen (`Ok(false)`) when no
/// Pokémon is there, there is nothing to place, or the event is refused (a lock; a prevention: Hide 'n' Sneak, Mist
/// Energy, Battle Cage, ...).
pub fn place(g: &mut Game, target: SlotRef, hp: i32, cause: Cause) -> R<bool> {
    if hp <= 0 || g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        return Ok(false);
    }
    let v = counters_view(g, target, hp, cause);
    if crate::engine::condition::refused(g, &v)?.is_some() {
        return Ok(false);
    }
    g.run_fx_unit(Effect::PlaceCounters { p: target.p, target, amount: hp, cause })?;
    Ok(true)
}

/// MoveCounters: one action moving counters, `pairs` of (from, to, HP) in the order the text or the player gives them.
/// Each pair moves as much as it can (the counters its source has then, id63). Returns what happened to each pair
/// (removed from the source, placed on the destination); nothing moves when a lock forbids the action.
pub fn move_counters(g: &mut Game, pairs: &[(SlotRef, SlotRef, i32)], cause: Cause) -> R<SVec<CounterMove, 16>> {
    let mut moves: SVec<CounterMove, 16> = SVec::new();
    let Some(&(first, _, _)) = pairs.first() else { return Ok(moves) };
    // The locks forbid the action (Patrat's Watchful Eye binds both players), asked once.
    let v = move_view(g, first, MoveEnd::From, pairs.iter().map(|x| x.2).sum(), cause);
    if crate::derived::event_locked(g, &v)?.is_some() {
        return Ok(moves);
    }
    // The damage each spot has as the pairs are carried out in order.
    let mut left: SVec<(SlotRef, i32), 16> = SVec::new();
    let damage_of = |g: &Game, left: &SVec<(SlotRef, i32), 16>, s: SlotRef| left.iter().find(|(x, _)| *x == s).map(|(_, d)| *d).unwrap_or(g.st.slot(s.p as usize, s.s).damage);
    let set = |left: &mut SVec<(SlotRef, i32), 16>, s: SlotRef, d: i32| match left.as_mut_slice().iter_mut().find(|(x, _)| *x == s) {
        Some(e) => e.1 = d,
        None => left.push((s, d)),
    };
    for &(from, to, hp) in pairs {
        if g.st.slot_pokemon(from.p as usize, from.s).is_none() || g.st.slot_pokemon(to.p as usize, to.s).is_none() {
            continue;
        }
        let removed = hp.min(damage_of(g, &left, from));
        if removed <= 0 {
            continue;
        }
        let vf = move_view(g, from, MoveEnd::From, removed, cause);
        if crate::derived::event_prevented(g, &vf)? {
            continue;
        }
        let vt = move_view(g, to, MoveEnd::To, removed, cause);
        let placed = if crate::derived::event_prevented(g, &vt)? { 0 } else { removed };
        let d = damage_of(g, &left, from);
        set(&mut left, from, d - removed);
        let d = damage_of(g, &left, to);
        set(&mut left, to, d + placed);
        moves.push(CounterMove { from, to, removed, placed });
    }
    if !moves.is_empty() {
        g.run_fx_unit(Effect::MoveCounters { p: cause.player, moves, cause })?;
    }
    Ok(moves)
}

// ---------------------------------------------------------------------------
// Consequences, applied by the events' reducer

/// The reducer of this module's events: what each does to the game. Moving counters off a Pokémon isn't healing it
/// ("healed this turn" untouched; id1995, id2254).
pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::PlaceCounters { target, amount, .. } => {
            g.st.players[target.p as usize].slots[target.s as usize].damage += amount.max(0);
        }
        Effect::MoveCounters { moves, .. } => {
            for m in moves.iter() {
                let s = &mut g.st.players[m.from.p as usize].slots[m.from.s as usize];
                s.damage = (s.damage - m.removed).max(0);
                g.st.players[m.to.p as usize].slots[m.to.s as usize].damage += m.placed;
            }
        }
        _ => {}
    }
    Ok(())
}
