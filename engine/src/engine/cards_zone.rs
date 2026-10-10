//! The card-movement events of events batch 7 (docs/design/events-design.md, sections 4 and 4.6; APR C-01, C-02, H):
//! Discard, PutIntoHand, PutIntoDeck and Draw, the recorded routines of the cards that aren't dispatched (Look, Reveal,
//! Shuffle, SetPrizes), and the physical relocation below every event.
//!
//! In the `engine::enter` pattern: the routine checks each card (it is still where it was chosen; the locks,
//! `derived::event_locked`), produces one event per action and zone carrying the cards that move (user decision D3; a
//! refused card stays where it is and the others move: "do as much as you can", APR II.A), applies the consequences in
//! [`reducer`] (the cards go to their owner's zone, APR C-01 / C-02, in the order the action takes them), and the
//! triggers over the event run per card (`spec::run::after_event`).
//!
//! - [`discard`]: cards from the hand or the deck (or looked at: the zone the look started from) to the discard pile (a
//!   Prism Star card to the Lost Zone). A card leaving a Pokémon or the Stadium leaving play is a LeavePlay
//!   (`engine::knockout`; user decision D1), not a Discard.
//! - [`put_into_hand`] / [`put_into_deck`]: cards from the deck, the discard pile, the Prizes or the cards looked at;
//!   "this card can't be put into your hand or deck from the discard pile" (Poké Vital A, Neutralization Zone) is a lock
//!   over them.
//! - [`draw`]: the top cards of the deck into the hand, as many as there are (draw as much as you can).
//! - [`reveal`], [`stage`] (Look), [`shuffle_deck`], [`set_prizes`]: no card reacts to them (user decision D2), so they
//!   are routines, not dispatched events; every ShowCards and ShuffleDeck prompt stays where it was (the replay's chance
//!   tape).
//!
//! [`relocate`] is the physical move (MoveCards' semantics for a list of cards), used by the events' reducers and by the
//! physical layers of the other events (`enter`, `knockout`, `attach`); [`settle`] is what follows every card move until
//! batch 8 narrows it: the derived layer is invalidated and the Ability-lock stamps re-synced (`passive::lock_sync`).

use crate::cause::Cause;
use crate::effects::{EffId, Effect};
use crate::game::{Game, R};
use crate::list::*;
use crate::spec::event::{DeckPosition, EventKind, EventView, RulesZone};
use crate::state::ListRef;
use crate::types::*;

// ---------------------------------------------------------------------------
// The physical layer

/// Put the cards just moved at the top of the destination: they were pushed onto its end, so lift them out first.
fn reorder_top(g: &mut Game, dst: ListRef, cards: &[CardId]) {
    let cur: SVec<CardId, 120> = {
        let mut v = SVec::new();
        for &c in g.lst(dst) {
            v.push(c);
        }
        v
    };
    let mut out: SVec<CardId, 120> = SVec::new();
    for &c in cards {
        if cur.contains(&c) {
            out.push(c);
        }
    }
    for &c in cur.iter() {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    g.lst_mut(dst).set_from(out.as_slice());
}

/// The physical move of `cards` from `src` to `dst`, in list order (below the events: no check, nothing dispatched). A
/// Prism Star card that would go to a discard pile goes to its owner's Lost Zone (first, as the move always did). With
/// `top` the cards end up on top of `dst` (a deck's top). When the cards leave a Pokémon's spot and no Pokémon card is
/// left there, the rest of the spot goes to the discard pile and the spot is reset.
pub fn relocate(g: &mut Game, src: ListRef, cards: &[CardId], dst: ListRef, top: bool) {
    if let ListRef::Discard(owner) = dst {
        let mut lost: SVec<CardId, 64> = SVec::new();
        let mut disc: SVec<CardId, 64> = SVec::new();
        for &c in cards {
            if g.st.cdef(c).has_tag(tag::PRISM_STAR) {
                lost.push(c);
            } else {
                disc.push(c);
            }
        }
        if !lost.is_empty() {
            g.move_cards_to(src, lost.as_slice(), ListRef::LostZone(owner));
        }
        if !disc.is_empty() {
            g.move_cards_to(src, disc.as_slice(), dst);
        }
    } else {
        g.move_cards_to(src, cards, dst);
    }
    if top {
        reorder_top(g, dst, cards);
    }
    if let ListRef::Slot(p, s) = src {
        clean_spot(g, p, s);
    }
}

/// A spot whose Pokémon cards all left: its other cards go to the discard pile (a Prism Star card to the Lost Zone), its
/// Energy list and Tools are emptied, and the spot is reset.
fn clean_spot(g: &mut Game, p: u8, s: crate::state::SlotId) {
    let (pu, su) = (p as usize, s);
    if !g.st.slot_pokemons(pu, su).is_empty() {
        return;
    }
    let rest: SVec<CardId, 60> = {
        let mut v = SVec::new();
        for c in g.st.slot(pu, su).cards.iter() {
            v.push(c);
        }
        v
    };
    // The Prism Star cards first, then the others (the order the move always used).
    for &c in rest.iter() {
        if g.st.cdef(c).has_tag(tag::PRISM_STAR) {
            g.move_card_to(ListRef::Slot(p, s), c, ListRef::LostZone(p));
        }
    }
    for &c in rest.iter() {
        if !g.st.cdef(c).has_tag(tag::PRISM_STAR) {
            g.move_card_to(ListRef::Slot(p, s), c, ListRef::Discard(p));
        }
    }
    g.st.players[pu].slots[su as usize].energies.clear();
    let tools: SVec<CardId, 4> = {
        let mut v = SVec::new();
        for c in g.st.slot(pu, su).tools.iter() {
            v.push(c);
        }
        v
    };
    for &t in tools.iter() {
        let dst = if g.st.cdef(t).has_tag(tag::PRISM_STAR) { ListRef::LostZone(p) } else { ListRef::Discard(p) };
        g.move_card_to(ListRef::Slot(p, s), t, dst);
    }
    if g.st.slot_pokemons(pu, su).is_empty() {
        crate::engine::game_effect::reset_empty_slot(&mut g.st.players[pu].slots[su as usize]);
    }
}

/// The physical move of a whole Pokémon spot to `dst` (the LeavePlay of a Pokémon): its Tools first, then every card of
/// the spot in order (a Prism Star card to the Lost Zone when they go to a discard pile), then the spot is reset.
pub fn relocate_spot(g: &mut Game, spot: crate::effects::SlotRef, dst: ListRef) {
    let src = spot.list();
    let tools: SVec<CardId, 4> = {
        let mut v = SVec::new();
        for c in g.st.players[spot.p as usize].slots[spot.s as usize].tools.iter() {
            v.push(c);
        }
        v
    };
    for &t in tools.iter() {
        g.move_card_to(src, t, dst);
    }
    if let ListRef::Discard(owner) = dst {
        let all: SVec<CardId, 120> = {
            let mut v = SVec::new();
            for &c in g.lst(src) {
                v.push(c);
            }
            v
        };
        let mut lost: SVec<CardId, 64> = SVec::new();
        let mut disc: SVec<CardId, 120> = SVec::new();
        for &c in all.iter() {
            if g.st.cdef(c).has_tag(tag::PRISM_STAR) {
                lost.push(c);
            } else {
                disc.push(c);
            }
        }
        if !lost.is_empty() {
            g.move_cards_to(src, lost.as_slice(), ListRef::LostZone(owner));
        }
        if !disc.is_empty() {
            g.move_cards_to(src, disc.as_slice(), dst);
        }
    } else {
        g.move_to(src, dst, None);
    }
    clean_spot(g, spot.p, spot.s);
}

/// What follows every card move (today's timing, B7; batch 8 narrows it to moves in and out of play and the Stadium):
/// the derived layer is stale and the Ability-lock stamps are re-synced.
pub fn settle(g: &mut Game) {
    g.derived.invalidate();
    crate::spec::passive::lock_sync(g);
}

/// [`relocate`] then [`settle`]: a physical move that stands for a whole move of cards in another event's consequences
/// (a Devolve's removed cards, a Swap, the Bench shrinking, Team Rocket's Energy discarded at a move).
pub fn move_physical(g: &mut Game, src: ListRef, cards: &[CardId], dst: ListRef) {
    relocate(g, src, cards, dst, false);
    settle(g);
}

/// Stage `cards` from `src` into the staging list `temp` (cards looked at, chosen to be put somewhere: never a zone, design
/// section 11 item 5). The staged cards remember the rules zone they came from (`CardInst::staged_from`).
pub fn stage(g: &mut Game, src: ListRef, cards: &[CardId], temp: ListRef) {
    relocate(g, src, cards, temp, false);
    settle(g);
}

// ---------------------------------------------------------------------------
// Event views

/// The rules zone of a list cards are taken from: a staged card's is the zone its look or search started from.
pub fn source_zone(g: &Game, from: ListRef, card: CardId) -> RulesZone {
    match crate::engine::enter::rules_zone_of(from) {
        Some(z) => z,
        None => g.st.cards[card as usize].staged_from.unwrap_or(RulesZone::Deck),
    }
}

/// The view of one card of a card-movement event (`kind` Discard, PutIntoHand, PutIntoDeck, Draw).
pub fn card_view(g: &Game, kind: EventKind, card: CardId, source: RulesZone, position: Option<DeckPosition>, cause: Cause) -> EventView {
    let dest = match kind {
        EventKind::Discard if g.st.cdef(card).has_tag(tag::PRISM_STAR) => Some(RulesZone::LostZone),
        EventKind::Discard => Some(RulesZone::Discard),
        EventKind::PutIntoHand | EventKind::Draw => Some(RulesZone::Hand),
        EventKind::PutIntoDeck => Some(RulesZone::Deck),
        _ => None,
    };
    EventView { card: Some(card), source: Some(source), dest, position, ..EventView::new(kind, cause, g.st.owner(card) as u8, crate::spec::event::whose_turn(g)) }
}

/// The cards an event of this module carries (and a LeavePlay of attached cards or of the Stadium), for the per-card
/// views of its triggers; `None` for an event about one Pokémon.
pub fn event_cards(e: &Effect) -> Option<&SVec<CardId, 64>> {
    match e {
        Effect::Discard { cards, .. } | Effect::PutIntoHand { cards, .. } | Effect::PutIntoDeck { cards, .. } | Effect::Draw { cards, .. } => Some(cards),
        Effect::LeavePlay { cards, how, .. } if !matches!(how, crate::effects::LeaveHow::KnockOut | crate::effects::LeaveHow::Effect | crate::effects::LeaveHow::BenchShrink) => Some(cards),
        _ => None,
    }
}

/// The view of card `card` of the card event `e`.
pub fn view_of(g: &Game, e: &Effect, card: CardId) -> Option<EventView> {
    Some(match *e {
        Effect::Discard { source, cause, .. } => card_view(g, EventKind::Discard, card, source, None, cause),
        Effect::PutIntoHand { source, cause, .. } => card_view(g, EventKind::PutIntoHand, card, source, None, cause),
        Effect::PutIntoDeck { source, position, cause, .. } => card_view(g, EventKind::PutIntoDeck, card, source, Some(position), cause),
        Effect::Draw { cause, .. } => card_view(g, EventKind::Draw, card, RulesZone::Deck, None, cause),
        Effect::LeavePlay { target, dest, cause, how, .. } => crate::engine::knockout::card_leave_view(g, target, card, how, dest, cause),
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// The routines

/// The cards of `cards` still in `from`, in order, minus those a lock forbids (`kind`'s per-card view, the locks first:
/// `derived::event_locked`). Returns the cards that move and how many were refused.
fn allowed(g: &mut Game, kind: EventKind, from: ListRef, cards: &[CardId], position: Option<DeckPosition>, cause: Cause) -> R<(SVec<CardId, 64>, usize)> {
    let mut out: SVec<CardId, 64> = SVec::new();
    let mut refused = 0;
    let lockable = crate::spec::passive::may_lock_event(g, cause.player as usize, kind);
    for &c in cards {
        if !g.lst(from).contains(&c) || out.contains(&c) {
            continue;
        }
        if lockable {
            let v = card_view(g, kind, c, source_zone(g, from, c), position, cause);
            if crate::derived::event_locked(g, &v)?.is_some() {
                refused += 1;
                continue;
            }
        }
        out.push(c);
    }
    Ok((out, refused))
}

/// The cards of `cards` in `from` grouped by owner, in first-seen order (one event per owner: their own zone).
fn by_owner(g: &Game, cards: &[CardId]) -> [SVec<CardId, 64>; 2] {
    let mut out: [SVec<CardId, 64>; 2] = [SVec::new(), SVec::new()];
    for &c in cards {
        let o = g.st.owner(c);
        if !out[o].contains(&c) {
            out[o].push(c);
        }
    }
    out
}

/// Discard: `cards` (in `from`: a hand, a deck, or the cards looked at) go to their owner's discard pile, by `cause`. The
/// outcome: every card moved (`Done`), some (`Partial`), none because each was refused (`Prevented`), nothing to move
/// (`Impossible`).
pub fn discard(g: &mut Game, from: ListRef, cards: &[CardId], cause: Cause) -> R<crate::spec::run::Outcome> {
    let mut moved = 0;
    let mut refused = 0;
    for group in by_owner(g, cards).iter() {
        if group.is_empty() {
            continue;
        }
        let (ok, r) = allowed(g, EventKind::Discard, from, group.as_slice(), None, cause)?;
        refused += r;
        if ok.is_empty() {
            continue;
        }
        moved += ok.len();
        let p = g.st.owner(ok.as_slice()[0]) as u8;
        let source = source_zone(g, from, ok.as_slice()[0]);
        g.run_fx_unit(Effect::Discard { p, cards: ok, from, source, cause })?;
    }
    Ok(crate::spec::run::Outcome::count(cards.len(), moved, refused))
}

/// PutIntoHand: `cards` (in `from`) go to their owner's hand, by `cause`.
pub fn put_into_hand(g: &mut Game, from: ListRef, cards: &[CardId], cause: Cause) -> R<crate::spec::run::Outcome> {
    let mut moved = 0;
    let mut refused = 0;
    for group in by_owner(g, cards).iter() {
        if group.is_empty() {
            continue;
        }
        let (ok, r) = allowed(g, EventKind::PutIntoHand, from, group.as_slice(), None, cause)?;
        refused += r;
        if ok.is_empty() {
            continue;
        }
        moved += ok.len();
        let p = g.st.owner(ok.as_slice()[0]) as u8;
        let source = source_zone(g, from, ok.as_slice()[0]);
        g.run_fx_unit(Effect::PutIntoHand { p, cards: ok, from, source, cause })?;
    }
    Ok(crate::spec::run::Outcome::count(cards.len(), moved, refused))
}

/// PutIntoDeck: `cards` (in `from`) go into their owner's deck at `position` (in the order given: a group shuffled before
/// it is put is shuffled by the caller, APR E-35), by `cause`.
pub fn put_into_deck(g: &mut Game, from: ListRef, cards: &[CardId], position: DeckPosition, cause: Cause) -> R<crate::spec::run::Outcome> {
    let mut moved = 0;
    let mut refused = 0;
    for group in by_owner(g, cards).iter() {
        if group.is_empty() {
            continue;
        }
        let (ok, r) = allowed(g, EventKind::PutIntoDeck, from, group.as_slice(), Some(position), cause)?;
        refused += r;
        if ok.is_empty() {
            continue;
        }
        moved += ok.len();
        let p = g.st.owner(ok.as_slice()[0]) as u8;
        let source = source_zone(g, from, ok.as_slice()[0]);
        g.run_fx_unit(Effect::PutIntoDeck { p, cards: ok, from, source, position, cause })?;
    }
    Ok(crate::spec::run::Outcome::count(cards.len(), moved, refused))
}

/// Draw: player `p` draws `n` cards (as many as the deck has; APR H), by `cause`.
pub fn draw(g: &mut Game, p: usize, n: usize, cause: Cause) -> R<crate::spec::run::Outcome> {
    let k = n.min(g.st.players[p].deck.len());
    if k == 0 {
        return Ok(if n == 0 { crate::spec::run::Outcome::Done } else { crate::spec::run::Outcome::Impossible });
    }
    let mut top: SVec<CardId, 64> = SVec::new();
    for &c in g.st.players[p].deck.as_slice()[..k.min(64)].iter() {
        top.push(c);
    }
    let from = ListRef::Deck(p as u8);
    let (ok, refused) = allowed(g, EventKind::Draw, from, top.as_slice(), None, cause)?;
    if !ok.is_empty() {
        g.run_fx_unit(Effect::Draw { p: p as u8, cards: ok, cause })?;
    }
    Ok(crate::spec::run::Outcome::count(n, ok.len(), refused))
}

/// Reveal: `n` cards are shown to player `to` (the ShowCards information prompt; nothing when there are none). No card
/// reacts to it (user decision D2).
pub fn reveal(g: &mut Game, to: usize, n: usize) {
    if n == 0 {
        return;
    }
    let id = g.player_id(to);
    g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", crate::prompts::PromptKind::ShowCards, crate::game::Cont::Noop);
}

/// Shuffle: player `p`'s deck is shuffled (the ShuffleDeck chance prompt, then a silent wait). No card reacts to it.
pub fn shuffle_deck(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", crate::prompts::PromptKind::ShuffleDeck, crate::game::Cont::ShuffleApply { p: p as u8 });
}

/// SetPrizes: `card` (in `from`) becomes player `p`'s Prize card in Prize position `index` (Bother-Bot's switch, user
/// decision D12). No card reacts to it.
pub fn set_prize(g: &mut Game, from: ListRef, card: CardId, p: usize, index: u8) {
    relocate(g, from, &[card], ListRef::Prize(p as u8, index), false);
    settle(g);
}

// ---------------------------------------------------------------------------
// Consequences, applied by the events' reducer

/// The cards move to their owner's zone (the order of the action); the derived layer and the lock stamps follow
/// (`Game::reduce_effect`: the kinds are in `derived::INVALIDATING_KINDS` and re-sync the locks).
pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::Discard { p, cards, from, .. } => relocate(g, from, cards.as_slice(), ListRef::Discard(p), false),
        Effect::PutIntoHand { p, cards, from, .. } => relocate(g, from, cards.as_slice(), ListRef::Hand(p), false),
        Effect::PutIntoDeck { p, cards, from, position, .. } => relocate(g, from, cards.as_slice(), ListRef::Deck(p), position == DeckPosition::Top),
        Effect::Draw { p, cards, .. } => relocate(g, ListRef::Deck(p), cards.as_slice(), ListRef::Hand(p), false),
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! The routines against hand-built boards.
    use super::*;
    use crate::cause::CauseKind;
    use crate::effects::SlotRef;
    use crate::spec::run::Outcome;
    use serde_json::json;

    const VITAL: &str = "Poké Vital A SFA 62";
    const MILOTIC: &str = "Milotic TWM 50";
    const WATER: &str = "Water Energy MEE 3";

    fn game(sc: serde_json::Value) -> Game {
        let mut names: Vec<&str> = Vec::new();
        for (n, k) in [(VITAL, 1), (MILOTIC, 4), ("Feebas TWM 49", 4)] {
            names.extend(std::iter::repeat(n).take(k));
        }
        while names.len() < 60 {
            names.push(WATER);
        }
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    fn copies(g: &Game, l: ListRef, name: &str) -> Vec<CardId> {
        let def = crate::carddb::def_by_full_name(name).unwrap();
        g.lst(l).iter().copied().filter(|c| g.st.cards[*c as usize].def == def).collect()
    }

    /// Poké Vital A: "This card can't be put into your hand or deck from the discard pile" is a lock over its PutIntoHand /
    /// PutIntoDeck from the discard pile, whoever puts it: it stays, the other cards move (APR II.A).
    #[test]
    fn poke_vital_a_stays_in_the_discard_pile() {
        let mut g = game(json!({"me": {"reset": true, "active": MILOTIC, "discard": [VITAL, WATER, WATER]}, "opp": {"reset": true, "active": MILOTIC}}));
        let me = g.st.active_player as usize;
        let disc = ListRef::Discard(me as u8);
        let vital = copies(&g, disc, VITAL)[0];
        let water = copies(&g, disc, WATER)[0];
        let mine = Cause::new(CauseKind::Trainer, None, me as u8);
        assert_eq!(put_into_hand(&mut g, disc, &[vital, water], mine).unwrap(), Outcome::Partial);
        assert!(g.st.players[me].discard.contains(vital) && g.st.players[me].hand.contains(water));
        // The opponent's effect is locked too (both players), into the deck as well.
        let theirs = Cause::new(CauseKind::Attack, None, 1 - me as u8);
        assert_eq!(put_into_deck(&mut g, disc, &[vital], DeckPosition::Bottom, theirs).unwrap(), Outcome::Prevented);
        assert!(g.st.players[me].discard.contains(vital));
        // Discarding isn't locked: from the hand it goes to the discard pile.
        assert_eq!(discard(&mut g, ListRef::Hand(me as u8), &[water], mine).unwrap(), Outcome::Done);
        assert!(g.st.players[me].discard.contains(water));
    }

    /// Cards go to their owner's zone (APR C-01, C-02), whoever's effect moves them.
    #[test]
    fn cards_go_to_their_owners_zone() {
        // (No Milotic in play: Mentally Calm would keep the Energy out of my hand.)
        let mut g = game(json!({"me": {"reset": true, "active": "Feebas TWM 49", "active_energy": [WATER]}, "opp": {"reset": true, "active": "Feebas TWM 49", "hand": [WATER]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let a = SlotRef::new(me, g.st.players[me].active);
        let energy = g.st.slot(me, a.s).energies.as_slice()[0];
        let theirs = Cause::new(CauseKind::Trainer, None, o as u8);
        // The opponent's effect puts my attached Energy into "the hand": mine.
        let out = crate::engine::knockout::leave_play_cards(&mut g, a, &[energy], RulesZone::Hand, theirs, None).unwrap();
        assert_eq!(out, Outcome::Done);
        assert!(g.st.players[me].hand.contains(energy) && !g.st.players[o].hand.contains(energy));
        // My effect discards a card of the opponent's hand: their discard pile.
        let card = g.st.players[o].hand.as_slice()[0];
        discard(&mut g, ListRef::Hand(o as u8), &[card], Cause::new(CauseKind::Trainer, None, me as u8)).unwrap();
        assert!(g.st.players[o].discard.contains(card));
    }

    /// Milotic's Mentally Calm: the opponent's Pokémon in play and their attached cards can't be put into the opponent's
    /// hand (a `Prevent` over LeavePlay into the hand, both halves); a discard isn't prevented.
    #[test]
    fn milotic_mentally_calm() {
        let mut g = game(json!({"me": {"reset": true, "active": MILOTIC}, "opp": {"reset": true, "active": MILOTIC, "active_energy": [WATER, WATER], "bench": [{"card": MILOTIC}]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let a = SlotRef::new(o, g.st.players[o].active);
        let e = g.st.slot(o, a.s).energies.as_slice().to_vec();
        let mine = Cause::new(CauseKind::Attack, None, me as u8);
        assert_eq!(crate::engine::knockout::leave_play_cards(&mut g, a, &[e[0]], RulesZone::Hand, mine, None).unwrap(), Outcome::Prevented);
        assert!(g.st.slot(o, a.s).energies.contains(e[0]));
        assert_eq!(crate::engine::knockout::leave_play_cards(&mut g, a, &[e[0]], RulesZone::Discard, mine, None).unwrap(), Outcome::Done);
        let b = SlotRef::new(o, g.st.players[o].bench.as_slice()[0]);
        assert!(!crate::engine::knockout::leave_play(&mut g, b, ListRef::Hand(o as u8), mine, NO_CARD).unwrap(), "the whole Pokémon can't be put into the hand");
        // The opponent's own effect is stopped too ("can't be put into your opponent's hand").
        let theirs = Cause::new(CauseKind::Trainer, None, o as u8);
        assert_eq!(crate::engine::knockout::leave_play_cards(&mut g, a, &[e[1]], RulesZone::Hand, theirs, None).unwrap(), Outcome::Prevented);
    }

    /// Draw: as many as the deck has (APR H: draw as much as you can).
    #[test]
    fn draw_as_much_as_there_is() {
        let mut g = game(json!({"me": {"reset": true, "active": MILOTIC, "deck_left": 2}, "opp": {"reset": true, "active": MILOTIC}}));
        let me = g.st.active_player as usize;
        let n = g.st.players[me].hand.len();
        assert_eq!(draw(&mut g, me, 3, Cause::new(CauseKind::Trainer, None, me as u8)).unwrap(), Outcome::Partial);
        assert_eq!(g.st.players[me].hand.len(), n + 2);
        assert_eq!(draw(&mut g, me, 1, Cause::new(CauseKind::Trainer, None, me as u8)).unwrap(), Outcome::Impossible);
    }
}
