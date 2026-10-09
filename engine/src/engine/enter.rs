//! The Pokémon events of events batch 2 (docs/design/events-design.md, sections 4, 4.2, 4.4, 4.5):
//! EnterPlay, Evolve, Devolve and Swap.
//!
//! Every path that does one of these actions calls its routine here. A routine checks the action (locks,
//! evolution rules, permissions, restrictions: the same functions legality calls), produces the event (an
//! `Effect` dispatched to the cards), applies the event's consequences ([`reducer`], the consequences table
//! of section 4.4), and then the triggers over the event run (`spec::run::after_event`). The physical
//! relocation (`Game::move_card_to`) stays below the events: no declaration listens to it.
//!
//! The routines:
//! - [`enter_play`]: a Pokémon card goes onto an empty spot (played from the hand by the rule, put there by
//!   an effect from any zone, set up). [`play_from_hand`] is the turn action of playing a Pokémon card.
//! - [`evolve`]: an Evolution card goes onto a Pokémon (the rule path: from the hand by the rule or Rare
//!   Candy; the effect path: put onto it by an effect).
//! - [`devolve`]: Evolution cards leave a Pokémon (an attack's effect, Strange Timepiece).
//! - [`swap`]: the Pokémon card in a spot is replaced by another (Transformation Tome, Palafin, Ogre's Mask).

use crate::cause::Cause;
use crate::effects::{k, EffId, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::*;
use crate::spec::event::*;
use crate::spec::passive::{self, LockWhile, Modifier, Passive};
use crate::state::ListRef;
use crate::types::*;

// ---------------------------------------------------------------------------
// The Pokémon-identity table (events design 4.5)

/// Which Pokémon a fact belongs to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bound {
    /// The Pokémon card: evolving makes a new Pokémon that doesn't have it (id2265); a `Swap` moves it to the
    /// new card (the same Pokémon, id2372).
    Card,
    /// The spot: evolving, devolving and swapping keep it.
    Slot,
}

/// One fact about a Pokémon in play and where it lives.
pub struct Fact {
    pub name: &'static str,
    pub bound: Bound,
    pub stored: &'static str,
}

/// Every fact the events keep or move, declared once. `swap_card_facts` moves the card-bound ones; the
/// slot-bound ones stay with the spot because they are stored on it.
pub const IDENTITY: &[Fact] = &[
    Fact { name: "moved to the Active Spot this turn", bound: Bound::Card, stored: "CardInst::moved_to_active_this_turn, Player::moved_to_active_this_turn / moved_from_active_to_bench_this_turn" },
    Fact { name: "damage taken last turn", bound: Bound::Card, stored: "CardInst::damage_taken_last_turn" },
    Fact { name: "Ability used this turn", bound: Bound::Card, stored: "player markers from the card (Once::PerTurn): the card's own Abilities, so a new card has none used" },
    Fact { name: "entered play this turn", bound: Bound::Slot, stored: "Slot::entered_turn" },
    Fact { name: "damage counters", bound: Bound::Slot, stored: "Slot::damage" },
    Fact { name: "attached Energy and Tools", bound: Bound::Slot, stored: "Slot::cards / energies / tools" },
    Fact { name: "healed this turn", bound: Bound::Slot, stored: "Slot::healed_this_turn" },
];

/// `Swap`: the card-bound facts of `old` move to `new` (the same Pokémon, id2372); the slot-bound ones stay.
fn swap_card_facts(g: &mut Game, p: usize, old: CardId, new: CardId) {
    crate::prefabs::transfer_pokemon_card_state(g, p, old, new);
}

// ---------------------------------------------------------------------------
// Source zone

/// The rules zone of a list (`None` for a staging list).
pub fn rules_zone_of(l: ListRef) -> Option<RulesZone> {
    Some(match l {
        ListRef::Deck(_) => RulesZone::Deck,
        ListRef::Hand(_) => RulesZone::Hand,
        ListRef::Discard(_) => RulesZone::Discard,
        ListRef::LostZone(_) => RulesZone::LostZone,
        ListRef::Stadium(_) => RulesZone::Stadium,
        // A Trainer card being played sits in the play area; a Fossil played as a Pokémon goes back to the
        // hand first.
        ListRef::Supporter(_) => RulesZone::Hand,
        ListRef::Prize(..) => RulesZone::Prizes,
        ListRef::Slot(..) | ListRef::SlotEnergies(..) => RulesZone::InPlay,
        ListRef::Temp(_) => return None,
    })
}

/// Where `card` physically is (a zone, or a staging list) and its rules zone: for a staged card, the zone its
/// search or look started from (`CardInst::staged_from`). A staged card is taken from the staging list it was
/// moved into (`CardInst::staged_in`), not from a card register holding a copy of it (a Pick's selection:
/// Grimsley's Move would otherwise leave the card in its looked-at cards too).
pub fn source_of(g: &Game, card: CardId) -> Option<(ListRef, RulesZone)> {
    if let Some(l) = g.st.locate(card) {
        return rules_zone_of(l).map(|z| (l, z));
    }
    let inst = &g.st.cards[card as usize];
    if let Some(i) = inst.staged_in {
        if g.temps[i as usize].as_slice().contains(&card) {
            debug_assert!(inst.staged_from.is_some(), "a staged card without its source zone");
            return Some((ListRef::Temp(i), inst.staged_from.unwrap_or(RulesZone::Deck)));
        }
    }
    for i in 0..g.temps.len() {
        if g.temps[i].as_slice().contains(&card) {
            let z = g.st.cards[card as usize].staged_from;
            debug_assert!(z.is_some(), "a staged card without its source zone");
            return Some((ListRef::Temp(i as u8), z.unwrap_or(RulesZone::Deck)));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Event views

/// The EnterPlay event of `card` onto `target`.
pub fn enter_view(g: &Game, card: CardId, target: SlotRef, source: RulesZone, mode: EnterMode, cause: Cause) -> EventView {
    EventView { source: Some(source), mode: Some(mode), card: Some(card), slot: Some(target), ..EventView::new(EventKind::EnterPlay, cause, target.p, g.st.active_player) }
}

/// The Evolve event of `card` (`None`: a card not chosen yet) onto the Pokémon in `target`; `None` when the
/// spot holds no Pokémon.
pub fn evolve_view(g: &Game, card: Option<CardId>, target: SlotRef, source: RulesZone, path: EvolvePath, cause: Cause) -> Option<EventView> {
    let base = g.st.slot_pokemon(target.p as usize, target.s)?;
    let slot = g.st.slot(target.p as usize, target.s);
    let p = target.p as usize;
    Some(EventView {
        source: Some(source),
        path: Some(path),
        card,
        base: Some(base),
        slot: Some(target),
        base_entered_this_turn: slot.entered_turn == g.st.turn,
        // The owner's first turn: the game's turn 1 or 2 when it is the owner's turn.
        owner_first_turn: g.st.turn <= 2 && g.st.active_player as usize == p,
        ..EventView::new(EventKind::Evolve, cause, target.p, g.st.active_player)
    })
}

// ---------------------------------------------------------------------------
// The checks (execution and legality call the same functions)

/// How far an Evolution card reaches.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    /// The next Stage: the card's "Evolves from" names the Pokémon.
    Next,
    /// Rare Candy: a Stage 2 card whose Stage 1 evolves from the Basic Pokémon, skipping the Stage 1.
    SkipStage1,
}

/// The one "can `card` evolve `base`" function, from printed data (the three copies before batch 2: the
/// play from the hand, Rare Candy's match, Grand Tree / Salvatore's name test). A permission that lifts
/// [`Limit::EvolvesFrom`] (Rainbow DNA) is asked separately ([`evolve_rules`]).
pub fn evolves_into(g: &Game, base: CardId, card: CardId, reach: Reach) -> bool {
    let b = g.st.cdef(base);
    let e = g.st.cdef(card);
    match reach {
        Reach::Next => {
            (b.stage < e.stage && b.name == e.evolves_from)
                || b.evolves_to.contains(&e.name)
                || b.evolves_to_stage.contains(&e.stage)
                || (!b.evolves_from_base.is_empty() && b.evolves_from_base.contains(&e.evolves_from))
        }
        Reach::SkipStage1 => crate::gen::stage1::ALL_STAGE1.iter().any(|(n, from)| *n == e.evolves_from && *from == b.name),
    }
}

/// Does a limit apply to the event?
fn limit_applies(v: &EventView, l: Limit) -> bool {
    match l {
        Limit::FirstTurn => v.owner_first_turn,
        Limit::BaseEnteredThisTurn => v.base_entered_this_turn,
        Limit::EvolvesFrom => false,
    }
}

fn limit_error(l: Limit) -> &'static str {
    match l {
        Limit::FirstTurn => "CANNOT_EVOLVE_ON_YOUR_FIRST_TURN",
        Limit::BaseEnteredThisTurn => "POKEMON_CANT_EVOLVE_THIS_TURN",
        Limit::EvolvesFrom => "INVALID_TARGET",
    }
}

fn spec_passives(g: &Game, c: CardId) -> &'static [Passive] {
    crate::cards::spec_for(g.st.cards[c as usize].def).map_or(&[], |s| s.passives)
}

fn spec_restricts(g: &Game, c: CardId) -> &'static [crate::spec::Restrict] {
    crate::cards::spec_for(g.st.cards[c as usize].def).map_or(&[], |s| s.restricts)
}

/// The conditions on a declaration's source (`LockWhile`), for the card `me` located at `at`.
pub(crate) fn while_ok(g: &Game, me: CardId, at: passive::Located, while_: &[LockWhile], event_card: Option<CardId>) -> bool {
    while_.iter().all(|w| match w {
        LockWhile::Active => g.st.active_pokemon(at.owner) == Some(me),
        LockWhile::HasTool => at.held.map_or(false, |h| !g.st.slot(h.p as usize, h.s).tools.is_empty()),
        LockWhile::CardIsSource => event_card == Some(me),
    })
}

/// Does the permission `ps` of card `me` lift `limit` for the event (`Modifier::Permit`)?
fn permit_lifts(g: &mut Game, me: CardId, ps: &Passive, v: &EventView, limit: Limit) -> R<bool> {
    let Modifier::Permit(pm) = &ps.modifier else { return Ok(false) };
    if !pm.lifts.contains(&limit) {
        return Ok(false);
    }
    let Some(at) = passive::locate(g, me, ps.origin) else { return Ok(false) };
    if !while_ok(g, me, at, pm.while_, v.card) || !pm.for_.eval(g, me, v)? {
        return Ok(false);
    }
    Ok(!passive::blocked(g, me, ps.origin, at, v.slot))
}

/// The declaration marker kind of a permission lifting `limit` (`effects::k::PERMIT_*`, set in the declaring
/// card's mask).
pub const fn limit_kind(l: Limit) -> u32 {
    match l {
        Limit::FirstTurn => k::PERMIT_FIRST_TURN,
        Limit::BaseEnteredThisTurn => k::PERMIT_BASE_ENTERED,
        Limit::EvolvesFrom => k::PERMIT_EVOLVES_FROM,
    }
}

/// Does some card of the game declare a permission lifting `limit` (a plain read)?
#[inline]
pub fn may_permit(g: &Game, limit: Limit) -> bool {
    g.kinds_present.has(limit_kind(limit))
}

/// Is `limit` lifted for the event by a permission in force (a card in play: a Pokémon's Ability, a Tool,
/// an Energy, the Stadium)?
pub fn permitted(g: &mut Game, v: &EventView, limit: Limit) -> R<bool> {
    if !may_permit(g, limit) {
        return Ok(false);
    }
    let mut cands: SVec<CardId, 120> = SVec::new();
    if let Some(s) = g.st.stadium_card() {
        cands.push(s);
    }
    for q in 0..2 {
        for s in g.st.players[q].in_play().iter() {
            let slot = g.st.slot(q, *s);
            for c in slot.cards.iter().chain(slot.tools.iter()) {
                cands.push(c);
            }
        }
    }
    for c in cands.iter().copied() {
        for ps in spec_passives(g, c) {
            if permit_lifts(g, c, ps, v, limit)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// The rule's limits on evolving (APR A-05) that apply to the event and that no permission lifts: the error of
/// the first, or `None`. Only a rule-path Evolve is subject to them; an effect ignores them (APR C-12) unless
/// its card restricts itself (`CardSpec::restricts`: Rare Candy, Grand Tree), which nothing lifts.
pub fn rule_limits(g: &mut Game, v: &EventView) -> R<Option<&'static str>> {
    if v.path != Some(EvolvePath::Rule) {
        return Ok(None);
    }
    for l in [Limit::FirstTurn, Limit::BaseEnteredThisTurn] {
        if limit_applies(v, l) && !permitted(g, v, l)? {
            return Ok(Some(limit_error(l)));
        }
    }
    Ok(None)
}

/// The cards of the event whose declarations bind it: its cause card, then its card (once).
fn event_cards(v: &EventView) -> SVec<CardId, 2> {
    let mut out: SVec<CardId, 2> = SVec::new();
    for c in [v.cause.card, v.card].into_iter().flatten() {
        if c != NO_CARD && !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

/// Does a card of the event (its cause card, its card) restrict it ("you can't use this card during your
/// first turn or on a Basic Pokémon that was put into play this turn")? No permission lifts a restriction
/// (id1144, id1815).
pub fn restricted(g: &mut Game, v: &EventView) -> R<Option<&'static str>> {
    for c in event_cards(v).iter().copied() {
        for r in spec_restricts(g, c) {
            let Some(l) = r.limits.iter().copied().find(|l| limit_applies(v, *l)) else { continue };
            if r.on.eval(g, c, v)? {
                return Ok(Some(limit_error(l)));
            }
        }
    }
    Ok(None)
}

/// Do the restrictions need a look (a card of the event declares one)? A plain read,
/// for legality's fast path.
pub fn may_be_restricted(g: &Game, v: &EventView) -> bool {
    g.kinds_present.has(k::DECLARES_RESTRICT) && [v.cause.card, v.card].iter().flatten().any(|c| *c != NO_CARD && !spec_restricts(g, *c).is_empty())
}

/// The limits of an Evolve event whose card isn't chosen yet (an effect asks before offering cards: Grand
/// Tree's Pokémon prompt): the rule's limits with the permissions in force (rule path), then the restrictions.
pub fn evolve_limits(g: &mut Game, v: &EventView) -> R<Option<&'static str>> {
    if let Some(code) = rule_limits(g, v)? {
        return Ok(Some(code));
    }
    restricted(g, v)
}

/// The evolution rules of an Evolve event, after the locks: the card evolves from the Pokémon (rule path;
/// unless a permission lifts it), the rule's limits (rule path; unless permissions lift them), the
/// restrictions (any path; nothing lifts them).
pub fn evolve_rules(g: &mut Game, v: &EventView, reach: Reach) -> R {
    let (Some(card), Some(base)) = (v.card, v.base) else { crate::bail!("INVALID_TARGET") };
    if v.path == Some(EvolvePath::Rule) && !evolves_into(g, base, card, reach) && !(reach == Reach::Next && permitted(g, v, Limit::EvolvesFrom)?) {
        crate::bail!("INVALID_TARGET");
    }
    if let Some(code) = rule_limits(g, v)? {
        crate::bail!(code);
    }
    if let Some(code) = restricted(g, v)? {
        crate::bail!(code);
    }
    Ok(())
}

/// [`evolve_rules`] answered from a plain read when no permission or restriction can matter (legality's
/// fast path); `None` when it needs the checked reads.
pub fn evolve_rules_fast(g: &Game, v: &EventView, reach: Reach) -> Option<bool> {
    let (Some(card), Some(base)) = (v.card, v.base) else { return Some(false) };
    let rule = v.path == Some(EvolvePath::Rule);
    if rule && !evolves_into(g, base, card, reach) {
        if reach == Reach::Next && may_permit(g, Limit::EvolvesFrom) {
            return None;
        }
        return Some(false);
    }
    if rule {
        for l in [Limit::FirstTurn, Limit::BaseEnteredThisTurn] {
            if limit_applies(v, l) {
                if may_permit(g, l) {
                    return None;
                }
                return Some(false);
            }
        }
    }
    if may_be_restricted(g, v) {
        return None;
    }
    Some(true)
}

/// Every check of an Evolve event: the locks, then [`evolve_rules`].
pub fn check_evolve(g: &mut Game, v: &EventView, reach: Reach) -> R {
    if let Some(code) = crate::derived::event_locked(g, v)? {
        crate::bail!(code);
    }
    evolve_rules(g, v, reach)
}

/// Every check of an EnterPlay event: an empty spot, the locks, the restrictions.
pub fn check_enter(g: &mut Game, v: &EventView) -> R {
    let Some(t) = v.slot else { crate::bail!("INVALID_TARGET") };
    if !g.st.slot(t.p as usize, t.s).cards.is_empty() {
        crate::bail!("INVALID_TARGET");
    }
    if let Some(code) = crate::derived::event_locked(g, v)? {
        crate::bail!(code);
    }
    if let Some(code) = restricted(g, v)? {
        crate::bail!(code);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Playing a Pokémon card from the hand (the turn action)

/// The event playing `card` from the hand onto `target` produces (a plain read: the card and the spot only):
/// a Basic Pokémon onto an empty spot enters play, an Evolution onto a Pokémon evolves it.
pub fn hand_play_view(g: &Game, p: usize, card: CardId, target: SlotRef) -> R<EventView> {
    let cause = Cause::rule(crate::cause::RuleWhich::Action, p as u8);
    let d = g.st.cdef(card);
    if d.stage == Stage::Basic as u8 && g.st.slot(target.p as usize, target.s).cards.is_empty() {
        return Ok(enter_view(g, card, target, RulesZone::Hand, EnterMode::Rule, cause));
    }
    match evolve_view(g, Some(card), target, RulesZone::Hand, EvolvePath::Rule, cause) {
        Some(v) => Ok(v),
        None => crate::bail!("INVALID_TARGET"),
    }
}

/// Play a Pokémon card from the hand onto `target` (the turn action): EnterPlay or Evolve by the rule.
pub fn play_from_hand(g: &mut Game, p: usize, card: CardId, target: SlotRef) -> R {
    let v = hand_play_view(g, p, card, target)?;
    match v.kind {
        EventKind::EnterPlay => {
            check_enter(g, &v)?;
            run_enter(g, card, target, EnterMode::Rule, v.cause)
        }
        _ => {
            check_evolve(g, &v, Reach::Next)?;
            run_evolve(g, card, target, EvolvePath::Rule, v.cause)
        }
    }
}

// ---------------------------------------------------------------------------
// The routines

/// EnterPlay by an effect (or a Fossil played as a Pokémon): `card` goes onto the empty spot `target` (its
/// owner's) from wherever it is, `mode` saying how. The locks and restrictions are checked first: a refused
/// event doesn't happen (`Ok(false)`; an effect does as much as it can), nor does it for a card that is
/// nowhere (already moved away).
pub fn enter_play(g: &mut Game, card: CardId, target: SlotRef, mode: EnterMode, cause: Cause) -> R<bool> {
    let Some((_, source)) = source_of(g, card) else { return Ok(false) };
    let v = enter_view(g, card, target, source, mode, cause);
    if check_enter(g, &v).is_err() {
        return Ok(false);
    }
    run_enter(g, card, target, mode, cause)?;
    Ok(true)
}

/// EnterPlay at setup: a starting Pokémon put down from the hand (the setup rules chose it; nothing locks it).
pub fn put_at_setup(g: &mut Game, card: CardId, target: SlotRef) -> R {
    run_enter(g, card, target, EnterMode::Setup, Cause::rule(crate::cause::RuleWhich::Setup, target.p))
}

fn run_enter(g: &mut Game, card: CardId, target: SlotRef, mode: EnterMode, cause: Cause) -> R {
    let Some((from, source)) = source_of(g, card) else { return Ok(()) };
    g.run_fx_unit(Effect::EnterPlay { p: target.p, card, target, from, source, mode, cause })
}

/// Evolve by an effect (or Rare Candy): `card` goes onto the Pokémon in `target` from wherever it is, by
/// `path`. The locks, evolution rules, permissions and restrictions are checked first (`reach`: how far the
/// card may reach); a refused event doesn't happen (`Ok(false)`).
pub fn evolve(g: &mut Game, card: CardId, target: SlotRef, path: EvolvePath, reach: Reach, cause: Cause) -> R<bool> {
    let Some((_, source)) = source_of(g, card) else { return Ok(false) };
    let Some(v) = evolve_view(g, Some(card), target, source, path, cause) else { return Ok(false) };
    if check_evolve(g, &v, reach).is_err() {
        return Ok(false);
    }
    run_evolve(g, card, target, path, cause)?;
    Ok(true)
}

fn run_evolve(g: &mut Game, card: CardId, target: SlotRef, path: EvolvePath, cause: Cause) -> R {
    let Some((from, source)) = source_of(g, card) else { return Ok(()) };
    let Some(v) = evolve_view(g, Some(card), target, source, path, cause) else { crate::bail!("INVALID_TARGET") };
    let base = v.base.unwrap_or(NO_CARD);
    g.run_fx_unit(Effect::Evolve { p: target.p, target, card, base, from, source, path, cause, base_entered_this_turn: v.base_entered_this_turn, owner_first_turn: v.owner_first_turn })
}

/// Devolve: the top `count` Evolution cards of the Pokémon in `target` go to `dest`, highest Stage first. An
/// attack's devolving (`atk`) can be prevented as an effect of the attack (the `DevolveProbe`). It counts as
/// entering play this turn (APR C-13); a Pokémon left with damage counters at least its HP is Knocked Out by
/// the state check that follows every action.
pub fn devolve(g: &mut Game, target: SlotRef, count: usize, dest: ListRef, cause: Cause, atk: Option<crate::effects::AtkBase>) -> R {
    if let Some(b) = atk {
        let (_, prevented) = g.run_fx(Effect::DevolveProbe { b })?;
        if prevented {
            return Ok(());
        }
    }
    // B6: the routine's shape stays until events batch 6: no lock check before the event, the consequences
    // (devolve_one) applied before the Devolve effect is dispatched, one event per Stage removed.
    for _ in 0..count {
        let (tp, ts) = (target.p as usize, target.s);
        if g.st.slot_pokemons(tp, ts).len() <= 1 {
            break;
        }
        let removed = devolve_one(g, target, dest, cause)?;
        if removed.is_empty() {
            break;
        }
        g.run_fx_unit(Effect::Devolve { p: target.p, target, removed, dest, cause })?;
    }
    Ok(())
}

/// `DEVOLVE_POKEMON(store, state, target, destination)`: the physical devolving of one Stage and its
/// consequences (4.4: Special Conditions and effects removed, entered this turn, once-per-turn reset).
fn devolve_one(g: &mut Game, t: SlotRef, dest: ListRef, cause: Cause) -> R<SVec<CardId, 3>> {
    let mut out: SVec<CardId, 3> = SVec::new();
    let (tp, ts) = (t.p as usize, t.s);
    let pokemons = g.st.slot_pokemons(tp, ts);
    let top = g.st.slot_pokemon(tp, ts);
    let top_def = top.map(|c| g.st.cdef(c));
    if let (Some(_), Some(d)) = (top, top_def) {
        if d.has_tag(tag::POKEMON_LV_X) {
            if pokemons.len() == 2 && pokemons.iter().any(|c| g.st.cdef(*c).stage == Stage::Basic as u8) {
                return Ok(out);
            }
            let cards: Vec<CardId> = pokemons.iter().copied().filter(|c| g.st.cdef(*c).name == d.name).collect();
            crate::prefabs::move_cards(g, t.list(), dest, &cards, NO_CARD)?;
            for c in cards.iter().rev().take(3) {
                out.push(*c);
            }
            crate::engine::condition::recover_all(g, t, cause, &[])?;
            let turn = g.st.turn;
            let slot = &mut g.st.players[tp].slots[ts as usize];
            crate::engine::game_effect::clear_effects(slot);
            slot.entered_turn = turn;
            return Ok(out);
        }
    }
    // CardTag.LEGEND is TAG_NAMES index 30.
    let special = top_def.map(|d| d.has_tag(tag::POKEMON_VUNION) || d.has_tag(30)).unwrap_or(false);
    if pokemons.len() > 1 && !special {
        if let Some(top) = top {
            // MOVE_CARD_TO: findCardList(card).moveCardTo(card, destination).
            if let Some(src) = g.st.locate(top) {
                g.move_card_to(src, top, dest);
            }
            out.push(top);
        }
        crate::engine::condition::recover_all(g, t, cause, &[])?;
        let turn = g.st.turn;
        let slot = &mut g.st.players[tp].slots[ts as usize];
        crate::engine::game_effect::clear_effects(slot);
        slot.entered_turn = turn;
        crate::prefabs::reset_once_per_turn_slot(g, t); // id317
    }
    Ok(out)
}

/// How the new card takes the old card's place in the spot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwapPlace {
    /// Where the new card lands (the top of the stack).
    Top,
    /// At the old card's index in the stack.
    OldIndex,
    /// At the bottom of the stack, the old card leaving after the new one arrived (the spot is never empty).
    Bottom,
}

/// Swap: the Pokémon card `old` in `target` is replaced by `new` (from wherever it is); `old` goes to `into`.
/// The card-bound facts move to the new card, the slot-bound ones stay (4.5; id2372). The physical moves are
/// `MoveCards` effects, as before.
pub fn swap(g: &mut Game, target: SlotRef, old: CardId, new: CardId, into: ListRef, place: SwapPlace, me: CardId, cause: Cause) -> R {
    // B6: the routine's shape stays until events batch 6: no lock check before the event, the physical moves and
    // the card-bound facts applied before the Swap effect is dispatched.
    let (p, s) = (target.p as usize, target.s);
    // Where the new card comes from, read before it moves (the event's source).
    let Some((src, source)) = source_of(g, new) else { return Ok(()) };
    let list = target.list();
    let old_index = g.st.slot(p, s).cards.index_of(old);
    crate::prefabs::move_cards(g, src, list, &[new], me)?;
    crate::prefabs::move_cards(g, list, into, &[old], me)?;
    match place {
        SwapPlace::Top => {}
        SwapPlace::OldIndex => {
            let slot = &mut g.st.players[p].slots[s as usize];
            if let (Some(ni), Some(oi)) = (slot.cards.index_of(new), old_index) {
                if ni != oi {
                    slot.cards.remove_at(ni);
                    let at = oi.min(slot.cards.len());
                    slot.cards.insert(at, new);
                }
            }
        }
        SwapPlace::Bottom => {
            let mut order: Vec<CardId> = vec![new];
            order.extend(g.st.slot(p, s).cards.iter().filter(|c| *c != new));
            g.st.players[p].slots[s as usize].cards = List::from_slice(&order);
        }
    }
    swap_card_facts(g, p, old, new);
    g.run_fx_unit(Effect::Swap { p: target.p, target, old, new, source, cause })
}

// ---------------------------------------------------------------------------
// Consequences (events design 4.4), applied by the events' reducer

/// The reducer of the Pokémon events: what each does to the game.
pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::EnterPlay { card, target, from, .. } => {
            if !g.st.slot(target.p as usize, target.s).cards.is_empty() {
                crate::bail!("INVALID_TARGET");
            }
            g.move_card_to(from, card, target.list());
            let turn = g.st.turn;
            let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
            slot.entered_turn = turn;
            Ok(())
        }
        Effect::Evolve { card, target, from, cause, .. } => {
            if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
                crate::bail!("INVALID_TARGET");
            }
            g.move_card_to(from, card, target.list());
            let turn = g.st.turn;
            let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
            slot.entered_turn = turn;
            slot.marker.remove_all_except_trainer_effects();
            evolution_consequences(g, target.p as usize, target, cause)
        }
        // Devolve and Swap act in their routines (the physical moves are MoveCards effects there).
        _ => Ok(()),
    }
}

/// What evolving does to the Pokémon: it keeps its Energy, Tools and damage counters, and loses its Special
/// Conditions (except the ones a card preserves: `CheckSpecialConditionRemoval`) and the effects on it; it is a
/// new Pokémon for once-per-turn Abilities (id317) and has none of the old card's card-bound facts (id2265).
/// Rare Candy counts as evolving (id1045).
fn evolution_consequences(g: &mut Game, p: usize, target: SlotRef, cause: Cause) -> R {
    use crate::markers::*;
    let (e, _) = g.run_fx(Effect::CheckSpecialConditionRemoval { p: p as u8, target, preserved: SVec::new() })?;
    let preserved = match e {
        Effect::CheckSpecialConditionRemoval { preserved, .. } => preserved,
        _ => SVec::new(),
    };
    // player.removePokemonEffects(target)
    g.st.players[p].marker.remove(KNOCKOUT_MARKER);
    g.st.players[p].marker.remove(CLEAR_KNOCKOUT_MARKER);
    // The Pokémon recovers from its Special Conditions, except the preserved ones (RemoveCondition events, by the
    // Evolve's cause).
    crate::engine::condition::recover_all(g, target, cause, preserved.as_slice())?;
    let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
    crate::engine::game_effect::clear_effects(slot);
    slot.marker.remove_all_except_trainer_effects();
    slot.board_effect.retain(|b| *b != BoardEffect::AbilityUsed as u8);
    crate::prefabs::reset_once_per_turn_slot(g, target);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A card staged in one list and copied into a card register with a lower index is taken from the staging
    /// list (Grimsley's Move: Scratch(0) holds the looked-at cards, the Pick's register a copy of the chosen one).
    #[test]
    fn a_staged_card_leaves_its_staging_list_not_a_copy() {
        let mut names: Vec<&str> = vec!["Charcadet PFL 19"; 4];
        names.extend(["Finizen TWM 59"; 4]);
        names.extend(["Fire Energy MEE 2"; 52]);
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &serde_json::json!({"me": {"reset": true, "hand_to_deck": true, "active": "Charcadet PFL 19"}, "opp": {"reset": true, "active": "Charcadet PFL 19"}})).unwrap();
        let me = g.st.active_player as usize;
        let card = g.st.players[me].deck.iter().find(|c| g.st.cdef(*c).is_pokemon()).unwrap();
        // The copy register is allocated first (the lower index), the staging list second.
        let copy = g.alloc_temp(&[]);
        let staging = g.alloc_temp(&[]);
        g.move_card_to(ListRef::Deck(me as u8), card, staging);
        g.lst_mut(copy).set_from(&[card]);
        assert_eq!(source_of(&g, card), Some((staging, RulesZone::Deck)));
        let s = g.st.players[me].bench.as_slice()[0];
        assert!(g.st.slot(me, s).cards.is_empty());
        assert!(enter_play(&mut g, card, SlotRef::new(me, s), EnterMode::Effect, Cause::rule(crate::cause::RuleWhich::Action, me as u8)).unwrap());
        assert!(!g.lst(staging).contains(&card), "the card left its staging list");
        assert_eq!(g.st.slot_pokemon(me, s), Some(card));
    }

    #[test]
    fn identity_table_declares_every_fact_once() {
        let mut names: Vec<&str> = IDENTITY.iter().map(|f| f.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), IDENTITY.len());
        assert!(IDENTITY.iter().any(|f| f.bound == Bound::Card) && IDENTITY.iter().any(|f| f.bound == Bound::Slot));
    }
}
