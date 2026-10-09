//! The attaching events of events batch 3 (docs/design/events-design.md, sections 4 and 4.3): Attach,
//! MoveEnergy and MoveTool.
//!
//! Every path that attaches an Energy or a Pokémon Tool card, or moves one from a Pokémon to another, calls its
//! routine here, in the `engine::enter` pattern: the routine checks the event (the same functions legality
//! calls), produces it (an `Effect` dispatched to the cards), applies its consequences in [`reducer`], and then
//! the triggers over the event run (`spec::run::after_event`; `Game::reduce_effect` re-stamps the Ability locks
//! before them when an attached card can change one, `spec::passive::lock_sync_attached`). The physical relocation (`Game::move_card_to`) stays below the events.
//!
//! - [`attach`]: an Energy or a Tool goes onto a Pokémon from a zone that isn't a Pokémon (the hand, the deck,
//!   the discard pile, the cards looked at or searched: the event's `source` is the zone the look started from,
//!   `CardInst::staged_from`). One event per card. A card already attached to a Pokémon is moved instead
//!   (id1653: moving isn't attaching).
//! - [`play_energy`] / the Tool play (`play.rs`): the turn actions, from the hand. Only the turn's Energy
//!   attachment is `manual` (APR C-09: effects don't use it up; no lock reads it).
//! - [`move_energy`] / [`move_tool`]: an attached card moves from one Pokémon to another (APR C-10). A card that
//!   can't be on the new Pokémon is discarded at the move (Team Rocket's Energy).
//!
//! "From the hand" is the event's `source`: a lock on `Attach & Source(Hand)` covers every way a card goes from
//! the hand onto a Pokémon, an Ability or an attack attaching it included (id25, id230), and never a move or an
//! attach from the deck (id1950).

use crate::cause::Cause;
use crate::effects::{EffId, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::*;
use crate::spec::event::*;
use crate::spec::passive;
use crate::state::ListRef;
use crate::types::*;

/// Is the card a Pokémon Tool (it goes among the Pokémon's Tools)?
#[inline]
pub fn is_tool(g: &Game, card: CardId) -> bool {
    let d = g.st.cdef(card);
    d.is_trainer() && d.trainer_type == TrainerType::Tool as u8
}

// ---------------------------------------------------------------------------
// Event views

/// The Attach event of `card` onto the Pokémon in `target`, from `source`. The event's owner is the card's owner
/// (whose hand or deck it comes from).
pub fn attach_view(g: &Game, card: CardId, target: SlotRef, source: RulesZone, manual: bool, cause: Cause) -> EventView {
    EventView { source: Some(source), manual, card: Some(card), slot: Some(target), ..EventView::new(EventKind::Attach, cause, g.st.owner(card) as u8, g.st.active_player) }
}

/// The MoveEnergy / MoveTool event of `card` from the Pokémon in `from` to the one in `to` (the event's spot). The
/// event's owner is the Pokémon's owner.
pub fn move_view(g: &Game, kind: EventKind, card: CardId, from: SlotRef, to: SlotRef, cause: Cause) -> EventView {
    EventView { source: Some(RulesZone::InPlay), card: Some(card), slot: Some(to), ..EventView::new(kind, cause, from.p, g.st.active_player) }
}

// ---------------------------------------------------------------------------
// The checks (execution and legality call the same functions)

/// The spot can receive the card (a plain read): a Pokémon is there; a Tool: it is the owner's Pokémon unless the
/// Tool attaches to the opponent's, and the Pokémon has room for it (one Tool per Pokémon unless its card says
/// otherwise: `max_tools`); an Energy: the owner's Pokémon (APR C-09: players only attach Energy to their own).
pub fn attach_target_ok(g: &Game, v: &EventView) -> R {
    let (Some(card), Some(t)) = (v.card, v.slot) else { crate::bail!("INVALID_TARGET") };
    let Some(pc) = g.st.slot_pokemon(t.p as usize, t.s) else { crate::bail!("INVALID_TARGET") };
    if is_tool(g, card) {
        if t.p != v.owner && !g.st.cdef(card).attaches_to_opponents_pokemon {
            crate::bail!("INVALID_TARGET");
        }
        if g.st.slot(t.p as usize, t.s).tools.len() >= g.st.cdef(pc).max_tools as usize {
            crate::bail!("POKEMON_TOOL_ALREADY_ATTACHED");
        }
    } else if t.p != v.owner {
        crate::bail!("INVALID_TARGET");
    }
    Ok(())
}

/// The Energy's own "this card can only be attached to ..." (`Modifier::AttachGuard`): the error when it refuses
/// the spot.
pub fn attach_guard(g: &mut Game, v: &EventView) -> R {
    let (Some(card), Some(t)) = (v.card, v.slot) else { return Ok(()) };
    if passive::attach_guard_refuses(g, card, t)? {
        crate::bail!("CANNOT_PLAY_THIS_CARD");
    }
    Ok(())
}

/// Every check of an Attach event: the spot, the locks, the card's own guard.
pub fn check_attach(g: &mut Game, v: &EventView) -> R {
    attach_target_ok(g, v)?;
    if let Some(code) = crate::derived::event_locked(g, v)? {
        crate::bail!(code);
    }
    attach_guard(g, v)
}

// ---------------------------------------------------------------------------
// The routines

/// The player's Energy attachment from the hand by the game rule (the turn action; `manual`): checked, then
/// attached. `uses_turn`: it is the turn's one attachment (not with an effect allowing more: Dragon's Wish). A
/// refused attachment is an error (the action is illegal).
pub fn play_energy(g: &mut Game, p: usize, card: CardId, target: SlotRef, uses_turn: bool) -> R {
    let cause = Cause::rule(crate::cause::RuleWhich::Action, p as u8);
    let v = attach_view(g, card, target, RulesZone::Hand, true, cause);
    check_attach(g, &v)?;
    if uses_turn {
        g.st.players[p].energy_played_turn = g.st.turn;
    }
    run_attach(g, card, target, true, cause)
}

/// The checks of playing the Tool `card` from the hand onto `target` (the Trainer play's own rules are checked
/// by the play).
pub fn check_tool_play(g: &mut Game, p: usize, card: CardId, target: SlotRef) -> R {
    let v = attach_view(g, card, target, RulesZone::Hand, false, Cause::rule(crate::cause::RuleWhich::Action, p as u8));
    check_attach(g, &v)
}

/// Attach by an effect: `card` goes onto the Pokémon in `target` from wherever it is (one event per card). The
/// checks come first: a refused attachment doesn't happen (`Ok(false)`; an effect does as much as it can), nor
/// does it for a card that is nowhere. A card already attached to a Pokémon is moved (MoveEnergy / MoveTool,
/// id1653).
pub fn attach(g: &mut Game, card: CardId, target: SlotRef, cause: Cause) -> R<bool> {
    let Some((_, source)) = crate::engine::enter::source_of(g, card) else { return Ok(false) };
    if source == RulesZone::InPlay {
        let Some((fp, fs)) = g.st.find_pokemon_slot(card) else { return Ok(false) };
        return move_attached(g, card, SlotRef::new(fp, fs), target, cause);
    }
    let v = attach_view(g, card, target, source, false, cause);
    if check_attach(g, &v).is_err() {
        return Ok(false);
    }
    run_attach(g, card, target, false, cause)?;
    Ok(true)
}

/// Produce the Attach event (checked by the caller).
pub(crate) fn run_attach(g: &mut Game, card: CardId, target: SlotRef, manual: bool, cause: Cause) -> R {
    let Some((from, source)) = crate::engine::enter::source_of(g, card) else { return Ok(()) };
    let p = g.st.owner(card) as u8;
    g.run_fx_unit(Effect::Attach { p, card, target, from, source, manual, cause })
}

/// An attached card moves from the Pokémon in `from` to the one in `to`: an Energy (MoveEnergy) or a Tool
/// (MoveTool). Anything else attached (never a pool card) moves physically.
pub fn move_attached(g: &mut Game, card: CardId, from: SlotRef, to: SlotRef, cause: Cause) -> R<bool> {
    if is_tool(g, card) {
        return move_tool(g, card, from, to, cause);
    }
    if g.st.cdef(card).is_energy() || g.st.slot(from.p as usize, from.s).energies.contains(card) {
        return move_energy(g, card, from, to, cause);
    }
    debug_assert!(false, "a moved attached card is neither an Energy nor a Tool");
    g.move_card_to(from.list(), card, to.list());
    Ok(true)
}

/// MoveEnergy: the Energy `card` attached to the Pokémon in `from` moves to the Pokémon in `to` (APR C-10). It
/// doesn't happen (`Ok(false)`) when the card isn't attached there any more, there is no Pokémon in `to`, the
/// two are the same Pokémon, or a lock forbids it.
pub fn move_energy(g: &mut Game, card: CardId, from: SlotRef, to: SlotRef, cause: Cause) -> R<bool> {
    let src = g.st.slot(from.p as usize, from.s);
    if from == to || !(src.cards.contains(card) || src.energies.contains(card)) || g.st.slot_pokemon(to.p as usize, to.s).is_none() {
        return Ok(false);
    }
    let v = move_view(g, EventKind::MoveEnergy, card, from, to, cause);
    if crate::derived::event_locked(g, &v)?.is_some() {
        return Ok(false);
    }
    g.run_fx_unit(Effect::MoveEnergy { p: from.p, card, from, to, cause })?;
    Ok(true)
}

/// MoveTool: the Tool `card` attached to the Pokémon in `from` moves to the Pokémon in `to`. It doesn't happen
/// when the Tool isn't there, `to` has no Pokémon or no room for another Tool (`max_tools`), or a lock forbids
/// it. No pool card moves a Tool today.
pub fn move_tool(g: &mut Game, card: CardId, from: SlotRef, to: SlotRef, cause: Cause) -> R<bool> {
    if from == to || !g.st.slot(from.p as usize, from.s).tools.contains(card) {
        return Ok(false);
    }
    let Some(pc) = g.st.slot_pokemon(to.p as usize, to.s) else { return Ok(false) };
    if g.st.slot(to.p as usize, to.s).tools.len() >= g.st.cdef(pc).max_tools as usize {
        return Ok(false);
    }
    let v = move_view(g, EventKind::MoveTool, card, from, to, cause);
    if crate::derived::event_locked(g, &v)?.is_some() {
        return Ok(false);
    }
    g.run_fx_unit(Effect::MoveTool { p: from.p, card, from, to, cause })?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// Consequences, applied by the events' reducer

/// The reducer of the attaching events: what each does to the game. "When attached" effects of Special Energy
/// and Tools are triggers over the event (`Event::On`), which run after it; the Ability locks are re-stamped after
/// it when an attached card can change them (`passive::lock_sync_attached`).
pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::Attach { card, target, from, .. } => {
            if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
                crate::bail!("INVALID_TARGET");
            }
            if from != target.list() {
                g.move_card_to(from, card, target.list());
            }
            put_attached(g, card, target);
            Ok(())
        }
        Effect::MoveEnergy { card, from, to, .. } => {
            g.move_card_to(from.list(), card, to.list());
            // A card that can't be on the new Pokémon is discarded at the move (Team Rocket's Energy).
            if passive::attach_guard_discards(g, card, to)? {
                crate::prefabs::move_cards(g, to.list(), ListRef::Discard(to.p), &[card], card)?;
            }
            Ok(())
        }
        Effect::MoveTool { card, from, to, .. } => {
            g.move_card_to(from.list(), card, to.list());
            put_attached(g, card, to);
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The card just put into the spot's cards is attached there: a Tool goes among the Tools, an Energy among the
/// Energy.
fn put_attached(g: &mut Game, card: CardId, target: SlotRef) {
    let tool = is_tool(g, card);
    let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
    if tool {
        slot.cards.remove(card);
        if !slot.tools.contains(card) {
            slot.tools.push(card);
        }
    } else if !slot.energies.contains(card) {
        slot.energies.push(card);
    }
}

#[cfg(test)]
mod tests {
    //! The attaching events against hand-built boards (the rules fixes no pool card reaches: every pool effect that
    //! attaches from the hand, deck or discard pile takes Basic Energy, and no pool card moves a Tool).
    use super::*;
    use crate::cause::CauseKind;
    use serde_json::json;

    const DURA: &str = "Duraludon PRE 69";
    const MEWTWO: &str = "Team Rocket's Mewtwo ex DRI 81";
    const ROCKET_ENERGY: &str = "Team Rocket's Energy ASC 217";
    const ENRICHING: &str = "Enriching Energy SSP 191";
    const IGNITION: &str = "Ignition Energy PFL 124";
    const CHARM: &str = "Sacred Charm PFL 93";
    const GENESECT: &str = "Genesect SFA 40";

    fn game(sc: serde_json::Value) -> Game {
        let mut names: Vec<&str> = Vec::new();
        for (n, k) in [(DURA, 4), (MEWTWO, 3), (ROCKET_ENERGY, 3), (ENRICHING, 1), (IGNITION, 3), (CHARM, 4), (GENESECT, 3)] {
            names.extend(std::iter::repeat(n).take(k));
        }
        while names.len() < 60 {
            names.push("Metal Energy MEE 8");
        }
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    /// A copy of `name` of player `p` in the list `l`.
    fn copy_in(g: &Game, p: usize, name: &str, l: ListRef) -> CardId {
        let def = crate::carddb::def_by_full_name(name).unwrap();
        g.lst(l).iter().copied().find(|c| g.st.cards[*c as usize].def == def && g.st.owner(*c) == p).unwrap()
    }

    fn ability(p: usize) -> Cause {
        Cause::new(CauseKind::Ability, None, p as u8)
    }

    fn bench(g: &Game, p: usize, i: usize) -> SlotRef {
        SlotRef::new(p, g.st.players[p].bench.as_slice()[i])
    }

    fn active(g: &Game, p: usize) -> SlotRef {
        SlotRef::new(p, g.st.players[p].active)
    }

    /// One Tool per Pokémon (unless its card says otherwise), for every way of attaching one: Farfetch'd's
    /// Impromptu Carrier attached its Tool without the check before events batch 3.
    #[test]
    fn one_tool_per_pokemon() {
        let mut g = game(json!({"me": {"reset": true, "active": DURA, "active_tool": CHARM, "bench": [{"card": DURA}], "deck_top": [CHARM]}, "opp": {"reset": true, "active": DURA}}));
        let me = g.st.active_player as usize;
        let charm = copy_in(&g, me, CHARM, ListRef::Deck(me as u8));
        let a = active(&g, me);
        assert!(!attach(&mut g, charm, a, ability(me)).unwrap(), "the Active Pokémon has a Tool");
        assert!(g.st.players[me].deck.contains(charm));
        assert_eq!(g.st.slot(me, a.s).tools.len(), 1);
        let b = bench(&g, me, 0);
        assert!(attach(&mut g, charm, b, ability(me)).unwrap());
        assert!(g.st.slot(me, b.s).tools.contains(charm) && !g.st.slot(me, b.s).cards.contains(charm));
        // Moving a Tool onto a Pokémon that has one doesn't happen either.
        let other = g.st.slot(me, a.s).tools.as_slice()[0];
        assert!(!move_tool(&mut g, other, a, b, ability(me)).unwrap());
    }

    /// Moving an attached Energy isn't attaching it (id1653): Enriching Energy draws nothing, and a lock on
    /// attaching from the hand (Genesect's ACE Nullifier) doesn't read a move.
    #[test]
    fn a_move_is_not_an_attach() {
        let mut g = game(json!({"me": {"reset": true, "active": DURA, "active_energy": [ENRICHING], "bench": [{"card": DURA}]},
            "opp": {"reset": true, "active": GENESECT, "active_tool": CHARM}}));
        let me = g.st.active_player as usize;
        let (a, b) = (active(&g, me), bench(&g, me, 0));
        let e = copy_in(&g, me, ENRICHING, a.list());
        let hand = g.st.players[me].hand.len();
        let v = move_view(&g, EventKind::MoveEnergy, e, a, b, ability(me));
        assert_eq!(crate::derived::event_locked(&mut g, &v).unwrap(), None);
        assert!(move_energy(&mut g, e, a, b, ability(me)).unwrap());
        g.settle().ok();
        assert_eq!(g.st.players[me].hand.len(), hand, "no draw");
        assert!(g.st.slot(me, b.s).energies.contains(e) && !g.st.slot(me, a.s).cards.contains(e));
    }

    /// "When you attach this card from your hand" fires for an effect attaching it from the hand (APR C-09), once
    /// per card, and not for one attaching it from the deck (id1950).
    #[test]
    fn from_hand_triggers_read_the_source_zone() {
        for (from_hand, drawn) in [(true, 3), (false, 0)] {
            let mut g = game(json!({"me": {"reset": true, "active": DURA, "hand": [ENRICHING]}, "opp": {"reset": true, "active": DURA}}));
            let me = g.st.active_player as usize;
            let e = copy_in(&g, me, ENRICHING, ListRef::Hand(me as u8));
            if !from_hand {
                g.move_card_to(ListRef::Hand(me as u8), e, ListRef::Deck(me as u8));
            }
            let hand = g.st.players[me].hand.len() as i32;
            let a = active(&g, me);
            assert!(attach(&mut g, e, a, ability(me)).unwrap());
            g.settle().ok();
            assert_eq!(g.st.players[me].hand.len() as i32 - hand, drawn, "from the hand: {from_hand}");
        }
    }

    /// Ignition Energy is discarded at the end of the turn it is attached by any route (its marker was set by the
    /// attach effect only, which the raw-move routes didn't produce).
    #[test]
    fn ignition_energy_marker_on_every_route() {
        let mut g = game(json!({"me": {"reset": true, "active": DURA, "discard": [IGNITION]}, "opp": {"reset": true, "active": DURA}}));
        let me = g.st.active_player as usize;
        let e = copy_in(&g, me, IGNITION, ListRef::Discard(me as u8));
        let a = active(&g, me);
        assert!(attach(&mut g, e, a, ability(me)).unwrap());
        g.settle().ok();
        let id = crate::markers::marker_id("IGNITION_ENERGY_MARKER").unwrap();
        assert!(g.st.players[me].marker.has(id));
    }

    /// Team Rocket's Energy can only be attached to a Team Rocket's Pokémon, and is discarded at the move when it
    /// is moved to another (card text; events design 4: "a card that can't be on the target is discarded at the
    /// move").
    #[test]
    fn rocket_energy_refused_and_discarded_at_the_move() {
        let mut g = game(json!({"me": {"reset": true, "active": MEWTWO, "active_energy": [ROCKET_ENERGY], "bench": [{"card": DURA}], "discard": [ROCKET_ENERGY]}, "opp": {"reset": true, "active": DURA}}));
        let me = g.st.active_player as usize;
        let (a, b) = (active(&g, me), bench(&g, me, 0));
        let from_discard = copy_in(&g, me, ROCKET_ENERGY, ListRef::Discard(me as u8));
        assert!(!attach(&mut g, from_discard, b, ability(me)).unwrap(), "not a Team Rocket's Pokémon");
        let e = copy_in(&g, me, ROCKET_ENERGY, a.list());
        assert!(move_energy(&mut g, e, a, b, ability(me)).unwrap());
        assert!(g.st.players[me].discard.contains(e), "discarded at the move");
        assert!(!g.st.slot(me, b.s).cards.contains(e) && !g.st.slot(me, b.s).energies.contains(e));
    }
}
