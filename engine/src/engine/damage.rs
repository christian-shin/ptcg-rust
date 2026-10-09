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
//! The Cause is the frame's (`spec::run::Frame::cause`) or the rule's; ops never pick one.

use crate::cause::Cause;
use crate::effects::{EffId, Effect, SlotRef};
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

// ---------------------------------------------------------------------------
// Consequences, applied by the events' reducer

/// The reducer of this module's events: what each does to the game.
pub fn reducer(g: &mut Game, id: EffId) -> R {
    if let Effect::PlaceCounters { target, amount, .. } = *g.e(id) {
        g.st.players[target.p as usize].slots[target.s as usize].damage += amount.max(0);
    }
    Ok(())
}
