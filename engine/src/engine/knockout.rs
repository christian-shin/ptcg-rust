//! The Knock Out events of events batch 6 (docs/design/events-design.md, sections 4 and 4.4; APR D, E): the state
//! check, KnockOut, LeavePlay, TakePrizes and the end of the game.
//!
//! - [`state_check`] (a routine, not a dispatched event: no card reacts to "a check"; user decision D5): runs where the
//!   Check State step always ran (after each action or prompt with nothing pending, after the attack, at the end of the
//!   turn, after the Checkup, inside a barrage attack). Its order: (1) every Pokémon in play whose damage reaches its HP
//!   or that an effect Knocks Out (`Slot::ko_pending`), both players, all Knocked Out at the same time; (2) the
//!   KnockOut event of each while all are still in play (a Pokémon Knocked Out at the same time still has its Ability:
//!   id2089 / n1623), its triggers after it (Maractus, Togekiss, the Prize adjustments, Heavy Baton, Little Grudge);
//!   (3) the LeavePlay of each to its owner's discard pile, with every attached card; (4) TakePrizes, the player whose
//!   turn is next first (id2239; rulings 754, 757); (5) the promotion (batch 5's ChangeActive, next-turn player first;
//!   never refused: it is the rule); (6) the end of the game ([`game_end`]), after one more round when the step 2
//!   effects made new Knock Outs (rulings 1577, 1584, 1403).
//! - [`by_effect`]: "is Knocked Out" / "Knock Out ..." by an effect: the event's preventions (`KnockOut &
//!   KoBy(Effect)`: Mist Energy against Annihilape's Destined Fight, id2427; id2222), then recorded on the Pokémon and
//!   Knocked Out at the next state check with the others (id2089: all effects resolve before the Knock Outs; id810;
//!   JP FAQ Shuppet: 処理の順番は、ワザの効果がすべて終わったあと、きぜつ処理となります; user decision D1). A Knock Out
//!   the state check finds is the rule's and is never prevented.
//! - [`leave_play`]: a Pokémon and every card attached to it leave play (a Knock Out's, by the rule; an effect's: Scoop
//!   Up Cyclone, a Fossil's discard, a Pokémon shuffled into the deck, the Bench shrinking). Its Special Conditions and
//!   effects go with it (no RemoveCondition event), the spot is reset.
//! - [`take_prizes`]: Prize cards to the taker's hand, as many as there are; with fewer left than the count all are
//!   taken, otherwise the player chooses.
//! - [`game_end`]: the winner and why (Prizes, no Pokémon, deck-out, a card's effect, the Tiebreaker's Prize advantage).

use crate::cause::{Cause, RuleWhich};
use crate::effects::*;
use crate::engine::phase::end_game;
use crate::game::{Cont, Game, OnComplete, R};
use crate::list::*;
use crate::markers::*;
use crate::prompts::*;
use crate::spec::event::{EventKind, EventView, KoBy, RulesZone};
use crate::state::*;
use crate::types::*;

/// Why the game ended (kept for traces and the cabt adapter).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
    /// A player took their last Prize card.
    Prizes,
    /// A player has no Pokémon in play.
    NoPokemon,
    /// A player can't draw at the start of their turn.
    DeckOut,
    /// A card's effect ends the game.
    CardEffect,
    /// A Tiebreaker game: a player has Prize advantage after everything resolved.
    Tiebreaker,
}

/// The game ends: `winner` (a player, or a draw) for `reason`.
pub fn game_end(g: &mut Game, winner: Winner, reason: EndReason) {
    if g.st.phase != GamePhase::Finished {
        g.st.end_reason = Some(reason);
    }
    end_game(g, winner);
}

/// The cause of a Knock Out the state check finds that no attack's damage and no effect made: the rule, for the Knocked
/// Out Pokémon's opponent (who takes the Prizes).
pub fn by_state_check(owner: usize) -> Cause {
    Cause::rule(RuleWhich::StateCheck, (1 - owner) as u8)
}

// ---------------------------------------------------------------------------
// Event views

/// The KnockOut event of the Pokémon in `target`, as predicates read it.
pub fn ko_view(g: &Game, target: SlotRef, ko_by: KoBy, cause: Cause) -> EventView {
    let damaged_active = ko_by == KoBy::AttackDamage && g.attack_that_damaged_knocked_out(target.p as usize, target).is_some();
    EventView {
        card: g.st.slot_pokemon(target.p as usize, target.s),
        slot: Some(target),
        ko_by: Some(ko_by),
        damaged_active,
        ..EventView::new(EventKind::KnockOut, cause, target.p, crate::spec::event::whose_turn(g))
    }
}

/// The rules zone of a destination list.
pub fn zone_of(dest: ListRef) -> RulesZone {
    match dest {
        ListRef::Hand(_) => RulesZone::Hand,
        ListRef::Deck(_) => RulesZone::Deck,
        ListRef::LostZone(_) => RulesZone::LostZone,
        _ => RulesZone::Discard,
    }
}

/// The LeavePlay event of the Pokémon in `target` going to `dest`.
pub fn leave_view(g: &Game, target: SlotRef, dest: ListRef, cause: Cause) -> EventView {
    EventView { card: g.st.slot_pokemon(target.p as usize, target.s), slot: Some(target), dest: Some(zone_of(dest)), ..EventView::new(EventKind::LeavePlay, cause, target.p, crate::spec::event::whose_turn(g)) }
}

/// The LeavePlay of one card that leaves play without its Pokémon (`LeaveHow::Attached`: a card attached to the Pokémon in
/// `target`, whose owner the event is about; `LeaveHow::Stadium`: the Stadium, no spot), as predicates read it.
pub fn card_leave_view(g: &Game, target: Option<SlotRef>, card: CardId, how: LeaveHow, dest: ListRef, cause: Cause) -> EventView {
    let (source, owner) = match (how, target) {
        (LeaveHow::Stadium, _) | (_, None) => (RulesZone::Stadium, g.st.owner(card) as u8),
        (_, Some(t)) => (RulesZone::InPlay, t.p),
    };
    let dest = if matches!(dest, ListRef::Discard(_)) && g.st.cdef(card).has_tag(tag::PRISM_STAR) { RulesZone::LostZone } else { zone_of(dest) };
    EventView { card: Some(card), slot: target, source: Some(source), dest: Some(dest), ..EventView::new(EventKind::LeavePlay, cause, owner, crate::spec::event::whose_turn(g)) }
}

/// The TakePrizes event of player `p` (`count` Prize cards).
pub fn prizes_view(g: &Game, p: usize, count: i32, cause: Cause) -> EventView {
    EventView { amount: count, dest: Some(RulesZone::Hand), ..EventView::new(EventKind::TakePrizes, cause, p as u8, crate::spec::event::whose_turn(g)) }
}

// ---------------------------------------------------------------------------
// The routines

/// A Knock Out by an effect ("is Knocked Out", "Knock Out ..."): the event's checks (a Pokémon there; the locks; the
/// preventions over `KnockOut & KoBy(Effect)`), then the Pokémon is recorded to be Knocked Out at the next state check
/// with every other Knock Out (user decision D1). Returns whether it is recorded.
pub fn by_effect(g: &mut Game, target: SlotRef, cause: Cause) -> R<bool> {
    if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        return Ok(false);
    }
    let v = ko_view(g, target, KoBy::Effect, cause);
    if crate::engine::condition::refused(g, &v)?.is_some() {
        return Ok(false);
    }
    let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
    if slot.ko_pending.is_none() {
        slot.ko_pending = Some(cause);
    }
    Ok(true)
}

/// LeavePlay by an effect: the Pokémon in `target` and every card attached to it go to `dest`, unless the event is
/// refused (a prevention: Mist Energy against an attack's removal; Milotic's "can't be put into the hand"). Returns
/// whether it left play.
pub fn leave_play(g: &mut Game, target: SlotRef, dest: ListRef, cause: Cause, source_card: CardId) -> R<bool> {
    if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        return Ok(false);
    }
    let v = leave_view(g, target, dest, cause);
    if crate::engine::condition::refused(g, &v)?.is_some() {
        return Ok(false);
    }
    let pokemon = g.st.slot_pokemon(target.p as usize, target.s).unwrap_or(NO_CARD);
    g.run_fx_unit(Effect::LeavePlay { p: target.p, target: Some(target), pokemon, dest, cause, how: LeaveHow::Effect, source_card, cards: SVec::new() })?;
    Ok(true)
}

/// LeavePlay by the rule (a Knock Out, the Bench shrinking): never refused.
pub fn leave_play_by_rule(g: &mut Game, target: SlotRef, dest: ListRef, cause: Cause, how: LeaveHow) -> R {
    if g.st.slot(target.p as usize, target.s).cards.is_empty() {
        return Ok(());
    }
    let pokemon = g.st.slot_pokemon(target.p as usize, target.s).unwrap_or(NO_CARD);
    g.run_fx_unit(Effect::LeavePlay { p: target.p, target: Some(target), pokemon, dest, cause, how, source_card: NO_CARD, cards: SVec::new() })
}

/// The list of `zone` of `owner` a card leaving play goes to (its owner's zone: APR C-01, C-02).
pub fn owner_list(owner: u8, zone: RulesZone) -> ListRef {
    match zone {
        RulesZone::Hand => ListRef::Hand(owner),
        RulesZone::Deck => ListRef::Deck(owner),
        RulesZone::LostZone => ListRef::LostZone(owner),
        _ => ListRef::Discard(owner),
    }
}

/// Is `card` attached to the Pokémon in `target` (its Energy, its Tools, or another card in its spot)?
fn attached_to(g: &Game, target: SlotRef, card: CardId) -> bool {
    let sl = g.st.slot(target.p as usize, target.s);
    (sl.cards.contains(card) || sl.energies.contains(card) || sl.tools.contains(card)) && g.st.slot_pokemon(target.p as usize, target.s) != Some(card)
}

/// LeavePlay of cards attached to the Pokémon in `target` (an Energy or a Tool discarded, put into the hand or into the
/// deck; user decision D1), for `zone` of their owner, by `cause`. One event per owner carrying the cards in order; each
/// card is checked (still attached, the locks, the preventions on the Pokémon: "prevent all effects of attacks" keeps an
/// attack's discard off it, ruling 1843; id2393 Hide 'n' Sneak) and a refused one stays (D3). An attack's Energy removal
/// waits for its damage (`window`: the attack's effect, and whether only an all-Energy removal waits): the call is queued
/// in the after-damage window and checked when it runs (RULES.md "Energy removed as an effect of an attack").
pub fn leave_play_cards(g: &mut Game, target: SlotRef, cards: &[CardId], zone: RulesZone, cause: Cause, window: Option<(EffId, bool)>) -> R<crate::spec::run::Outcome> {
    use crate::spec::run::Outcome;
    if let Some((atk, energy_only)) = window {
        if !cards.is_empty() && g.after_damage_open(atk) && (!energy_only || cards.iter().all(|c| g.st.cdef(*c).is_energy())) {
            let mut cs: SVec<CardId, 64> = SVec::new();
            for &c in cards {
                cs.push(c);
            }
            g.push_after_damage(atk, crate::game::AfterDmgStep::LeaveCards { target, cards: cs, zone, cause });
            return Ok(Outcome::Done);
        }
    }
    let mut asked = 0;
    let mut moved = 0;
    let mut refused = 0;
    for owner in 0..2u8 {
        let mut ok: SVec<CardId, 64> = SVec::new();
        for &c in cards {
            if g.st.owner(c) as u8 != owner || ok.contains(&c) {
                continue;
            }
            asked += 1;
            if !attached_to(g, target, c) {
                continue;
            }
            let dest = owner_list(owner, zone);
            let v = card_leave_view(g, Some(target), c, LeaveHow::Attached, dest, cause);
            if crate::engine::condition::refused(g, &v)?.is_some() {
                refused += 1;
                continue;
            }
            ok.push(c);
        }
        if ok.is_empty() {
            continue;
        }
        moved += ok.len();
        g.run_fx_unit(Effect::LeavePlay { p: target.p, target: Some(target), dest: owner_list(owner, zone), cause, how: LeaveHow::Attached, source_card: NO_CARD, pokemon: NO_CARD, cards: ok })?;
    }
    Ok(Outcome::count(asked, moved, refused))
}

/// LeavePlay of cards attached to the Pokémon in `target` by the rule (the Retreat Cost paid, APR A-03): never refused,
/// like the rule's other LeavePlays (a cost is paid as a whole).
pub fn leave_play_cards_by_rule(g: &mut Game, target: SlotRef, cards: &[CardId], zone: RulesZone, cause: Cause) -> R {
    for owner in 0..2u8 {
        let mut ok: SVec<CardId, 64> = SVec::new();
        for &c in cards {
            if g.st.owner(c) as u8 == owner && !ok.contains(&c) && attached_to(g, target, c) {
                ok.push(c);
            }
        }
        if !ok.is_empty() {
            g.run_fx_unit(Effect::LeavePlay { p: target.p, target: Some(target), dest: owner_list(owner, zone), cause, how: LeaveHow::Attached, source_card: NO_CARD, pokemon: NO_CARD, cards: ok })?;
        }
    }
    Ok(())
}

/// LeavePlay of the Stadium `card` for `zone` of its owner (discarded by an effect; user decision D1), by `cause`. The locks
/// are asked (no `Prevent` protects a Stadium: it isn't a Pokémon). Returns whether it left play.
pub fn leave_play_stadium(g: &mut Game, card: CardId, zone: RulesZone, cause: Cause) -> R<bool> {
    if g.st.stadium_card() != Some(card) {
        return Ok(false);
    }
    let owner = g.st.owner(card) as u8;
    let dest = owner_list(owner, zone);
    let v = card_leave_view(g, None, card, LeaveHow::Stadium, dest, cause);
    if crate::derived::event_locked(g, &v)?.is_some() {
        return Ok(false);
    }
    let mut cs: SVec<CardId, 64> = SVec::new();
    cs.push(card);
    g.run_fx_unit(Effect::LeavePlay { p: owner, target: None, dest, cause, how: LeaveHow::Stadium, source_card: NO_CARD, pokemon: NO_CARD, cards: cs })?;
    Ok(true)
}

/// TakePrizes of the Prize cards `prizes` (indices) of player `p` into their hand (the event; no card prevents or locks
/// taking Prizes).
pub fn take_prizes_chosen(g: &mut Game, p: usize, prizes: &[u8], cause: Cause) -> R {
    if prizes.is_empty() {
        return Ok(());
    }
    let mut v: SVec<u8, 6> = SVec::new();
    for &i in prizes {
        if !v.contains(&i) && v.len() < 6 {
            v.push(i);
        }
    }
    g.run_fx_unit(Effect::TakePrizes { p: p as u8, prizes: v, cause })
}

/// TakePrizes of `count` Prize cards by player `p`: all of them without a choice when there are no more than that left,
/// otherwise the ChoosePrize prompt (the event follows the answer).
pub fn take_prizes(g: &mut Game, p: usize, count: i32, cause: Cause) -> R {
    let left = g.st.players[p].prize_left() as i32;
    if count <= 0 || left <= 0 {
        return Ok(());
    }
    if count >= left {
        let ix = first_prizes(g, p, left as usize);
        return take_prizes_chosen(g, p, ix.as_slice(), cause);
    }
    prize_prompt(g, p, count, cause);
    Ok(())
}

/// The first `count` Prize cards of `p` that are there (an automatic take).
fn first_prizes(g: &Game, p: usize, count: usize) -> SVec<u8, 6> {
    let mut v = SVec::new();
    for i in 0..g.st.players[p].prize_count {
        if !g.st.players[p].prizes[i as usize].is_empty() && v.len() < count {
            v.push(i);
        }
    }
    v
}

/// The ChoosePrize prompt for `count` of `p`'s Prize cards (secret unless the Prizes are face up).
fn prize_prompt(g: &mut Game, p: usize, count: i32, cause: Cause) {
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_PRIZE_CARD",
        PromptKind::ChoosePrize {
            count: count as u8,
            blocked: SVec::new(),
            use_opponent_prizes: false,
            allow_cancel: false,
            is_secret: !g.st.players[p].prize_public[0],
            destination: Some(ListRef::Hand(p as u8)),
            face_down_only: false,
        },
        Cont::TakePrizes { p: p as u8, cause: cause.pack() },
    );
}

/// The ChoosePrize answer: the TakePrizes event of the chosen cards.
pub fn take_prizes_cont(g: &mut Game, p: u8, cause: [u8; 4], res: Res) -> R {
    if let Res::Prizes(ix) = res {
        take_prizes_chosen(g, p as usize, ix.as_slice(), Cause::unpack(cause))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The state check

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckStage {
    /// Resume after KO number `idx` was announced (its effect is `ko_fx`).
    AfterKo,
    /// Resume after Knock Out number `idx` took its Pokémon out of play (effect `ko_fxs[idx]`).
    AfterRemove,
    AfterBenchSize,
    AfterPrize,
    AfterActive,
}

#[derive(Clone, Copy, Debug)]
pub struct CheckFrame {
    pub stage: CheckStage,
    pub oc: OnComplete,
    pub kos: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>,
    /// Per Knock Out: its damage reached its HP (otherwise only an effect Knocks it Out).
    pub by_hp: SVec<bool, { crate::state::MAX_SLOT_REFS }>,
    /// The Pokémon whose Knock Out was announced in this round.
    pub announced: SVec<CardId, 16>,
    pub idx: u8,
    pub ko_fx: EffId,
    /// The announced Knock Out effects (retained until the Pokémon left play).
    pub ko_fxs: SVec<EffId, 16>,
    /// The Prize cards each player takes for this round's Knock Outs.
    pub prizes: [i32; 2],
    /// Prize prompts to open: (player, count).
    pub prize_prompts: SVec<(u8, i32), 4>,
    pub active_prompts: SVec<u8, 2>,
}

/// The state check (APR D): see the module documentation.
pub fn state_check(g: &mut Game, oc: OnComplete) -> R {
    if !matches!(g.st.phase, GamePhase::PlayerTurn | GamePhase::Attack | GamePhase::BetweenTurns) {
        return on_complete(g, oc);
    }
    let (kos, by_hp) = find_ko_pokemons(g)?;
    let mut announced = SVec::new();
    for k in kos.iter() {
        if let Some(c) = g.st.slot_pokemon(k.p as usize, k.s) {
            announced.push(c);
        }
    }
    let f = CheckFrame {
        stage: CheckStage::AfterKo,
        oc,
        kos,
        by_hp,
        announced,
        idx: 0,
        ko_fx: 0,
        ko_fxs: SVec::new(),
        prizes: [0, 0],
        prize_prompts: SVec::new(),
        active_prompts: SVec::new(),
    };
    ko_loop(g, f)
}

pub fn on_complete(g: &mut Game, oc: OnComplete) -> R {
    match oc {
        OnComplete::None => Ok(()),
        OnComplete::AfterEndTurn { p } => crate::engine::phase::after_end_turn(g, p as usize),
        OnComplete::InitNextTurn => {
            if g.st.phase != GamePhase::Finished {
                crate::engine::phase::init_next_turn(g)
            } else {
                Ok(())
            }
        }
    }
}

/// Every Pokémon in play to Knock Out now, player 0's then player 1's, Active first: its damage reaches its HP (`by_hp`)
/// or an effect Knocks it Out (`Slot::ko_pending`). An undamaged Pokémon is at 0 HP only when an effect lowers HP: with
/// no card reacting to CheckHp in the game its HP isn't read (`PTCG_VERIFY_CACHE=1` reads it and asserts).
#[allow(clippy::type_complexity)]
fn find_ko_pokemons(g: &mut Game) -> R<(SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>, SVec<bool, { crate::state::MAX_SLOT_REFS }>)> {
    let mut out = SVec::new();
    let mut by_hp = SVec::new();
    let hp_fixed = !g.kinds_present.has(k::CHECK_HP) && g.copy_sessions.is_empty() && !g.trace_effects;
    for p in 0..2 {
        for s in g.st.players[p].in_play().iter() {
            // forEachPokemon skips slots without a Pokémon card (e.g. a Bench
            // slot holding only an Energy moved there by Team Rocket's Zapdos).
            if g.st.slot_pokemon(p, *s).is_none() {
                continue;
            }
            let pending = g.st.slot(p, *s).ko_pending.is_some();
            // An undamaged Pokémon can't be at 0 HP in a legality trial (no effect lowers HP to 0 there) or when no card
            // changes HP; its HP isn't read.
            let skip = g.st.slot(p, *s).damage == 0 && (g.trial || hp_fixed);
            if skip && !crate::game::verify_cache() {
                if pending {
                    out.push(SlotRef::new(p, *s));
                    by_hp.push(false);
                }
                continue;
            }
            let hp = crate::derived::hp(g, p, *s)?;
            let dead = g.st.slot(p, *s).damage >= hp;
            if skip {
                assert!(!dead, "an undamaged Pokémon at 0 HP although no card changes HP");
            }
            if dead || pending {
                out.push(SlotRef::new(p, *s));
                by_hp.push(dead);
            }
        }
    }
    Ok((out, by_hp))
}

/// `winConditionMet`: `check_winner` would end the game or start a Tiebreaker (a player has no Pokémon in
/// play, an Active spot waiting for a promotion from the Bench does not count, or no Prize cards left).
fn win_condition_met(g: &Game) -> bool {
    (0..2).any(|i| {
        let pl = &g.st.players[i];
        let no_pokemon = pl.slots[pl.active as usize].cards.is_empty() && !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty());
        no_pokemon || pl.prizes[..pl.prize_count as usize].iter().all(|l| l.is_empty())
    })
}

/// `knockOutPendingBeforeWinner`: a Knock Out effect at step 2 (Maractus JTG's Explosive Needle) can put counters
/// on a Pokémon that is then Knocked Out in a further round. Every effect has to resolve before the winner is
/// determined, so that round's Knock Outs and Prizes count before the game ends (rulings 1577, 1584, 1403).
fn knock_out_pending_before_winner(g: &mut Game, f: &CheckFrame) -> R<bool> {
    if !win_condition_met(g) {
        return Ok(false);
    }
    for k in find_ko_pokemons(g)?.0.iter() {
        let c = g.st.slot_pokemon(k.p as usize, k.s);
        if !c.map_or(false, |c| f.announced.contains(&c)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// How the Pokémon in `t` is Knocked Out and by what: by damage from the opponent's attack in progress (its damage
/// reached its HP and the attack damaged it: the attack, the Attacking Pokémon as its card); by an effect (the recorded
/// cause); otherwise by the rule (counters, a Special Condition, an HP change).
fn attribution(g: &Game, t: SlotRef, by_hp: bool) -> (KoBy, Cause) {
    let owner = t.p as usize;
    if by_hp && g.knocked_out_by_attack_damage(owner, t).is_some() {
        if let Some(la) = g.last_attack {
            return (KoBy::AttackDamage, Cause::attack(la.p, la.pokemon, la.attack));
        }
    }
    if let Some(c) = g.st.slot(owner, t.s).ko_pending {
        return (KoBy::Effect, c);
    }
    (KoBy::Other, by_state_check(owner))
}

/// Every Knock Out is announced while all Pokémon are still in play (a Pokémon Knocked
/// Out at the same time still has its Ability: ruling 1623), then they leave play and
/// the Prizes are counted (R7F-16).
fn ko_loop(g: &mut Game, mut f: CheckFrame) -> R {
    while (f.idx as usize) < f.kos.len() {
        let t = *f.kos.get(f.idx as usize).unwrap();
        let by_hp = *f.by_hp.get(f.idx as usize).unwrap();
        let (ko_by, cause) = attribution(g, t, by_hp);
        let id = g.new_fx(Effect::KnockOut { p: t.p, target: t, prize_count: 1, prize_base: 1, ko_by, cause });
        g.reduce_effect(id)?;
        f.ko_fx = id;
        if g.has_prompts() {
            f.stage = CheckStage::AfterKo;
            g.wait_prompt(Cont::CheckState(f));
            return Ok(());
        }
        f.ko_fxs.push(id);
        f.idx += 1;
    }
    f.idx = 0;
    remove_loop(g, f)
}

fn remove_loop(g: &mut Game, mut f: CheckFrame) -> R {
    while (f.idx as usize) < f.ko_fxs.len() {
        let id = *f.ko_fxs.get(f.idx as usize).unwrap();
        if !g.prevented(id) {
            if let Effect::KnockOut { p, target, .. } = *g.e(id) {
                // The Knock Out's LeavePlay is the rule's (APR D step 3): never prevented.
                if g.st.slot_pokemon(target.p as usize, target.s).is_some() {
                    leave_play_by_rule(g, target, ListRef::Discard(p), by_state_check(p as usize), LeaveHow::KnockOut)?;
                }
            }
            if g.has_prompts() {
                f.stage = CheckStage::AfterRemove;
                g.wait_prompt(Cont::CheckState(f));
                return Ok(());
            }
        }
        finish_ko(g, &mut f);
    }
    after_kos(g, f)
}

fn finish_ko(g: &mut Game, f: &mut CheckFrame) {
    let id = *f.ko_fxs.get(f.idx as usize).unwrap();
    if !g.prevented(id) {
        if let Effect::KnockOut { p, prize_count, .. } = *g.e(id) {
            // Prize reductions (Legacy Energy, Lillie's Pearl, ...) never go below 0 (R7F-13, ruling 1745).
            f.prizes[1 - p as usize] += prize_count.max(0);
        }
    }
    g.release_fx(id);
    f.idx += 1;
}

fn after_kos(g: &mut Game, mut f: CheckFrame) -> R {
    let (e, _) = g.run_fx(Effect::CheckTableState { bench_sizes: [5, 5] })?;
    let sizes = match e {
        Effect::CheckTableState { bench_sizes } => bench_sizes,
        _ => [5, 5],
    };
    handle_bench_size_change(g, sizes);
    if g.has_prompts() {
        f.stage = CheckStage::AfterBenchSize;
        g.wait_prompt(Cont::CheckState(f));
        return Ok(());
    }
    after_bench_size(g, f)
}

fn after_bench_size(g: &mut Game, mut f: CheckFrame) -> R {
    if g.st.phase == GamePhase::Finished {
        return Ok(());
    }
    f.prize_prompts = choose_prize_cards(g, &mut f)?;
    f.idx = 0;
    prize_loop(g, f)
}

fn prize_loop(g: &mut Game, mut f: CheckFrame) -> R {
    if (f.idx as usize) < f.prize_prompts.len() {
        let (p, count) = *f.prize_prompts.get(f.idx as usize).unwrap();
        f.idx += 1;
        prize_prompt(g, p as usize, count, by_state_check(1 - p as usize));
        if g.has_prompts() {
            f.stage = CheckStage::AfterPrize;
            g.wait_prompt(Cont::CheckState(f));
            return Ok(());
        }
        return prize_loop(g, f);
    }
    if g.st.phase == GamePhase::Finished {
        return on_complete(g, f.oc);
    }
    // A player has no Prize cards left: the game is decided now (check_winner
    // counts both players' win conditions) unless both took their last Prize card
    // at the same time: then the new Active Pokémon are promoted and their effects
    // resolve before the winner is determined (R7F-12; rulings 820, 1584).
    let taken: [bool; 2] = [0, 1].map(|i: usize| g.st.players[i].prizes[..g.st.players[i].prize_count as usize].iter().all(|l| l.is_empty()));
    if (taken[0] || taken[1]) && !(taken[0] && taken[1]) {
        if knock_out_pending_before_winner(g, &f)? {
            return state_check(g, f.oc);
        }
        return check_winner(g, f.oc);
    }
    f.active_prompts = SVec::new();
    for p in next_turn_player_order(g) {
        let pl = &g.st.players[p];
        let has_active = !pl.slots[pl.active as usize].cards.is_empty();
        let has_bench = pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty());
        if !has_active && has_bench {
            f.active_prompts.push(p as u8);
        }
    }
    f.idx = 0;
    active_loop(g, f)
}

fn active_loop(g: &mut Game, mut f: CheckFrame) -> R {
    if (f.idx as usize) < f.active_prompts.len() {
        let p = *f.active_prompts.get(f.idx as usize).unwrap();
        f.idx += 1;
        let id = g.player_id(p as usize);
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.prompt(
            id,
            "CHOOSE_NEW_ACTIVE_POKEMON",
            PromptKind::ChoosePokemon {
                player_type: PlayerType::BottomPlayer,
                slots,
                min: 1,
                max: 1,
                allow_cancel: false,
                blocked: SVec::new(),
            },
            Cont::ChooseActive { p },
        );
        if g.has_prompts() {
            f.stage = CheckStage::AfterActive;
            g.wait_prompt(Cont::CheckState(f));
            return Ok(());
        }
        return active_loop(g, f);
    }
    if knock_out_pending_before_winner(g, &f)? {
        return state_check(g, f.oc);
    }
    check_winner(g, f.oc)?;
    g.st.bench_size_change_handled = false;
    Ok(())
}

pub fn resume(g: &mut Game, mut f: CheckFrame) -> R {
    match f.stage {
        CheckStage::AfterKo => {
            f.ko_fxs.push(f.ko_fx);
            f.idx += 1;
            ko_loop(g, f)
        }
        CheckStage::AfterRemove => {
            finish_ko(g, &mut f);
            remove_loop(g, f)
        }
        CheckStage::AfterBenchSize => after_bench_size(g, f),
        CheckStage::AfterPrize => prize_loop(g, f),
        CheckStage::AfterActive => active_loop(g, f),
    }
}

/// The promotion after the Knock Outs: the chosen Benched Pokémon goes to the empty Active Spot (batch 5's ChangeActive,
/// by the rule: never refused).
pub fn choose_active_cont(g: &mut Game, p: u8, res: Res) -> R {
    let p = p as usize;
    let sel = res.slots();
    if sel.len() != 1 {
        crate::bail!("ILLEGAL_ACTION");
    }
    let bench_index = g.st.players[p].bench_index_of(sel[0].s);
    let active = g.st.players[p].active;
    let bi = match bench_index {
        Some(i) if sel[0].p as usize == p && g.st.slot(p, active).cards.is_empty() => i,
        _ => crate::bail!("ILLEGAL_ACTION"),
    };
    let new_active = g.st.players[p].bench.as_slice()[bi];
    if !crate::engine::change_active::promote(g, p, new_active)? {
        crate::bail!("ILLEGAL_ACTION");
    }
    Ok(())
}

fn opponent_has_no_pokemon_in_play(g: &Game, taker: usize) -> bool {
    let o = &g.st.players[1 - taker];
    if !o.slots[o.active as usize].cards.is_empty() {
        return false;
    }
    !o.bench.iter().any(|b| !o.slots[*b as usize].cards.is_empty())
}

/// The player whose turn would be next takes Prizes first and promotes first
/// when both have Pokémon Knocked Out at the same time (rulings 754, 757).
fn next_turn_player_order(g: &Game) -> [usize; 2] {
    let next = if g.st.active_player == 0 { 1 } else { 0 };
    [next, 1 - next]
}

/// The TakePrizes of this round's Knock Outs, the next-turn player first: taken at once when the player takes all that
/// are left or the opponent has no Pokémon in play, otherwise asked (the returned prompts).
fn choose_prize_cards(g: &mut Game, f: &mut CheckFrame) -> R<SVec<(u8, i32), 4>> {
    let mut prompts = SVec::new();
    let mut took_last_prize = false;
    for i in next_turn_player_order(g) {
        let mut count = f.prizes[i];
        if count == 0 {
            continue;
        }
        let cause = by_state_check(1 - i);
        let left = g.st.players[i].prize_left() as i32;
        // Taking the last Prize cards does not end the game here: every effect
        // resolves and check_winner counts both players' win conditions
        // (R7F-12; rulings 234, 820, 1403, 1584).
        if count >= left && left > 0 {
            let ix = first_prizes(g, i, left as usize);
            take_prizes_chosen(g, i, ix.as_slice(), cause)?;
            took_last_prize = true;
            continue;
        }
        if count > 0 && opponent_has_no_pokemon_in_play(g, i) {
            let ix = first_prizes(g, i, count as usize);
            take_prizes_chosen(g, i, ix.as_slice(), cause)?;
            continue;
        }
        if count > left {
            count = left;
        }
        if count > 0 {
            prompts.push((i as u8, count));
        }
    }
    if took_last_prize {
        return Ok(SVec::new());
    }
    Ok(prompts)
}

/// The winner, once every effect has resolved (APR E): a player with no Prize cards left wins, a player with no
/// Pokémon in play loses; both at once: the one who met more conditions wins, the same number starts Sudden Death
/// (R7F-12; rulings 234, 820, 1403).
pub fn check_winner(g: &mut Game, oc: OnComplete) -> R {
    let mut points = [0; 2];
    let mut prizes_out = false;
    for i in 0..2 {
        let pl = &g.st.players[i];
        // No Pokémon in play (an Active spot waiting for a promotion from the Bench is not a loss).
        if pl.slots[pl.active as usize].cards.is_empty() && !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            points[1 - i] += 1;
        }
        if pl.prizes[..pl.prize_count as usize].iter().all(|l| l.is_empty()) {
            points[i] += 1;
            prizes_out = true;
        }
    }
    if points[0] > 0 && points[1] > 0 && points[0] == points[1] {
        return initiate_sudden_death(g);
    }
    if points[0] + points[1] == 0 {
        // A Tiebreaker game is over as soon as a player has Prize advantage: fewer Prize cards remaining than
        // the opponent, after everything has resolved (rulings 567, 580).
        if g.st.is_sudden_death {
            let (a, b) = (g.st.players[0].prize_left(), g.st.players[1].prize_left());
            if a != b {
                game_end(g, if a < b { WINNER_P1 } else { WINNER_P2 }, EndReason::Tiebreaker);
            }
        }
        return on_complete(g, oc);
    }
    let winner = if points[0] > points[1] {
        WINNER_P1
    } else if points[1] > points[0] {
        WINNER_P2
    } else {
        WINNER_DRAW
    };
    game_end(g, winner, if prizes_out { EndReason::Prizes } else { EndReason::NoPokemon });
    on_complete(g, oc)
}

fn handle_bench_size_change(g: &mut Game, sizes: [u8; 2]) {
    if g.st.bench_size_change_handled {
        return;
    }
    for p in 0..2 {
        let size = sizes[p] as usize;
        while g.st.players[p].bench.len() < size {
            let s = g.st.players[p].alloc_slot();
            g.st.players[p].slots[s as usize].is_public = true;
            g.st.players[p].bench.push(s);
            touch();
        }
        if g.st.players[p].bench.len() == size {
            continue;
        }
        // Remove empty slots, starting from the right side.
        let pl = &g.st.players[p];
        let len = pl.bench.len();
        let mut empty: SVec<SlotId, 8> = SVec::new();
        for i in (0..len).rev() {
            let s = pl.bench.as_slice()[i];
            if len - empty.len() > size && pl.slots[s as usize].cards.is_empty() {
                empty.push(s);
            }
        }
        if len - empty.len() <= size {
            let pl = &mut g.st.players[p];
            for i in (0..pl.bench.len()).rev() {
                let s = pl.bench.as_slice()[i];
                if empty.contains(&s) {
                    pl.bench.remove_at(i);
                    touch();
                    pl.free_slot(s);
                }
            }
            continue;
        }
        // More Pokémon than the new size: the player discards some.
        let count = (len - empty.len() - size) as u8;
        let mut mask = 0u16;
        for s in empty.iter() {
            mask |= 1 << *s;
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DISCARD",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: count, max: count, allow_cancel: false, blocked: SVec::new() },
            Cont::BenchShrink { p: p as u8, empty: mask },
        );
    }
    g.st.bench_size_change_handled = true;
}

/// The Bench shrank (a Stadium left play): the chosen Benched Pokémon leave play to the discard pile by the rule
/// (LeavePlay, never refused), and the empty spots and theirs are removed from the Bench.
pub fn bench_shrink_cont(g: &mut Game, p: u8, empty: u16, res: Res) -> R {
    let pu = p as usize;
    let chosen: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }> = match res {
        Res::Slots(s) => s,
        _ => SVec::new(),
    };
    let mut i = g.st.players[pu].bench.len();
    while i > 0 {
        i -= 1;
        let s = g.st.players[pu].bench.as_slice()[i];
        let selected = empty & (1 << s) != 0 || chosen.iter().any(|t| t.p == p && t.s == s);
        if !selected {
            continue;
        }
        leave_play_by_rule(g, SlotRef::new(pu, s), ListRef::Discard(p), Cause::rule(RuleWhich::StateCheck, p), LeaveHow::BenchShrink)?;
        let pl = &mut g.st.players[pu];
        if let Some(j) = pl.bench.position(&s) {
            pl.bench.remove_at(j);
            touch();
        }
        pl.free_slot(s);
    }
    Ok(())
}

/// `initiateSuddenDeath`: a Tiebreaker game is a new game (Advanced Player's Rulebook I-E; rulings 234, 820, 1403,
/// 1487). Every zone, the tools and the Supporter back to the deck (PokemonCardList.moveTo semantics, so attached
/// energies are pushed twice), a fresh Player for everything else (Pokémon slots, Prize lists, markers, per-turn and
/// per-game flags), the card instance flags and the last attacks reset, shuffle, then flip for the first player.
fn initiate_sudden_death(g: &mut Game) -> R {
    for p in 0..2u8 {
        let pl = &g.st.players[p as usize];
        let mut slot_lists: Vec<ListRef> = vec![ListRef::Slot(p, pl.active)];
        for b in pl.bench.iter() {
            slot_lists.push(ListRef::Slot(p, *b));
        }
        for l in slot_lists.iter() {
            let (sp, ss) = match *l {
                ListRef::Slot(a, b) => (a as usize, b),
                _ => continue,
            };
            let tools: Vec<CardId> = g.st.slot(sp, ss).tools.iter().collect();
            for t in tools {
                g.move_card_to(*l, t, ListRef::Deck(p));
            }
        }
        let pl = &g.st.players[p as usize];
        let mut lists: Vec<ListRef> = slot_lists;
        lists.push(ListRef::Discard(p));
        for i in 0..pl.prize_count {
            lists.push(ListRef::Prize(p, i));
        }
        lists.push(ListRef::Hand(p));
        lists.push(ListRef::LostZone(p));
        lists.push(ListRef::Stadium(p));
        lists.push(ListRef::Supporter(p));
        for l in lists {
            g.move_to(l, ListRef::Deck(p), None);
        }
        let old = &g.st.players[p as usize];
        let (id, deck) = (old.id, old.deck);
        let cards: Vec<CardId> = deck.iter().collect();
        g.st.players[p as usize] = Player::new(id);
        g.st.players[p as usize].deck = deck;
        touch();
        for c in cards {
            let inst = &mut g.st.cards[c as usize];
            inst.moved_to_active_this_turn = false;
            inst.extra_prizes = false;
            inst.strafe_used = false;
            inst.discarded_stadium_card = false;
            inst.damage_taken_last_turn = 0;
        }
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p });
    }
    g.st.last_attack = None;
    g.st.player_last_attack = [None, None];
    g.st.player_last_attack_turn = [0, 0];
    let id = g.player_id(0);
    g.prompt(id, "SETUP_WHO_BEGINS_FLIP", PromptKind::CoinFlip, Cont::SuddenDeathCoin);
    Ok(())
}

/// `setupSuddenDeathGame`.
pub fn setup_sudden_death_game(g: &mut Game, first: u8) -> R {
    g.st.active_player = first;
    g.st.turn = 0;
    g.st.phase = GamePhase::Setup;
    g.st.is_sudden_death = true;
    crate::engine::setup::start(g)
}

// ---------------------------------------------------------------------------
// Consequences, applied by the events' reducer

/// The reducer of this module's events: what each does to the game.
pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::KnockOut { .. } => knock_out(g, id),
        Effect::LeavePlay { .. } => leave(g, id),
        Effect::TakePrizes { p, prizes, .. } => {
            let p = p as usize;
            for &i in prizes.iter() {
                g.move_to(ListRef::Prize(p as u8, i), ListRef::Hand(p as u8), None);
                g.st.players[p].prizes_taken += 1;
                g.st.players[p].prizes_taken_this_turn += 1;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The KnockOut's consequences (before the Pokémon leaves play): the Prize cards it is worth (a Rule Box's more, the
/// adjustments its triggers and passives made), the delayed "discard an Energy from the Attacking Pokémon" of a
/// Pokémon Knocked Out by damage from an attack, the records of a Knock Out during the opponent's turn.
fn knock_out(g: &mut Game, id: EffId) -> R {
    let Effect::KnockOut { p, target, ko_by, .. } = *g.e(id) else { return Ok(()) };
    let p = p as usize;
    let Some(card) = g.st.slot_pokemon(target.p as usize, target.s) else { return Ok(()) };
    let d = g.st.cdef(card);
    let mut extra = 0;
    if d.has_tag(tag::POKEMON_EX) || d.has_tag(tag::POKEMON_V) || d.has_tag(tag::POKEMON_VSTAR) || d.has_tag(tag::POKEMON_EX_LOWER) || d.has_tag(tag::POKEMON_GX) {
        extra += 1;
    }
    if d.has_tag(tag::POKEMON_SV_MEGA) || d.has_tag(tag::TAG_TEAM) || d.has_tag(tag::DUAL_LEGEND) {
        extra += 1;
    }
    if d.has_tag(tag::POKEMON_VMAX) || d.has_tag(tag::POKEMON_VUNION) {
        extra += 2;
    }
    if let Effect::KnockOut { prize_count, prize_base, .. } = g.e_mut(id) {
        *prize_count += extra;
        *prize_base += extra;
    }
    let by_attack = ko_by == KoBy::AttackDamage;
    // "If this Pokémon is Knocked Out by damage from an attack during your opponent's next turn, discard an Energy from
    // the Attacking Pokémon": an effect of the Knocked Out Pokémon's earlier attack (B6-OLD -> batch 7: its discard is
    // the attack-effect probe).
    let (armed, pending, g_attack, g_source, g_owner) = {
        let ts = g.st.slot(target.p as usize, target.s);
        (
            ts.discard_attacker_energy_if_ko_next_turn,
            ts.discard_attacker_energy_if_ko_next_turn_pending,
            ts.discard_attacker_energy_if_ko_attack,
            ts.discard_attacker_energy_if_ko_source_card,
            ts.discard_attacker_energy_if_ko_attacker,
        )
    };
    if armed && !pending && by_attack {
        if let (Some(attack), Some(source_card), Some(owner)) = (g_attack, g_source, g_owner) {
            let prize_taker = 1 - p;
            // "The Attacking Pokémon" is the Pokémon that used the attack, wherever it is by now (ruling 460);
            // nothing happens when it left play.
            let attacker_slot = g.attacker_of_knock_out(p).and_then(|a| a.1);
            let energy: Vec<CardId> = match attacker_slot {
                Some(sl) => g.st.slot(prize_taker, sl.s).cards.iter().filter(|c| g.st.cdef(*c).is_energy()).collect(),
                None => Vec::new(),
            };
            if energy.len() == 1 {
                crate::engine::game_effect::little_grudge_discard(g, owner as usize, prize_taker, attack, source_card, attacker_slot.unwrap(), &energy)?;
            } else if energy.len() > 1 {
                let sl = attacker_slot.unwrap();
                let mut slots = SVec::new();
                let mut o = MoveOpts { allow_cancel: false, min: 1, max: Some(1), ..Default::default() };
                match g.st.players[prize_taker].bench.iter().position(|b| *b == sl.s) {
                    Some(bi) => {
                        slots.push(SlotType::Bench as u8);
                        for i in 0..g.st.players[prize_taker].bench.len() {
                            if i != bi {
                                o.blocked_from.push(CardTarget::new(PlayerType::TopPlayer, SlotType::Bench, i as u8));
                            }
                        }
                    }
                    None => slots.push(SlotType::Active as u8),
                }
                let pid = g.player_id(p);
                g.prompt(
                    pid,
                    "CHOOSE_ENERGIES_TO_DISCARD",
                    PromptKind::DiscardEnergy { player_type: PlayerType::TopPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
                    Cont::LittleGrudge { owner, prize_taker: prize_taker as u8, attack, source_card, target: sl },
                );
            }
        }
    }
    let owner = p;
    let attacker = 1 - p;
    let during_opp_turn = matches!(g.st.phase, GamePhase::PlayerTurn | GamePhase::Attack) && g.st.active_player as usize == attacker;
    if during_opp_turn {
        g.st.players[owner].pokemon_knocked_out_during_opponents_last_turn = true;
        let def_id = g.st.cards[card as usize].def;
        g.st.players[owner].pokemon_knocked_out_last_turn_entries.push(def_id);
        g.st.players[owner].pokemon_knocked_out_last_turn_by_attack.push(by_attack);
    }
    if by_attack {
        g.st.players[owner].pokemon_knocked_out_by_attack_during_opponents_last_turn = true;
    }
    Ok(())
}

/// The LeavePlay's consequences: every card of the Pokémon and every attached card goes to its owner's `dest` (a Prism
/// Star card to the Lost Zone), the Pokémon's Special Conditions and effects go with it, the spot is reset (its pending
/// Knock Out with it), and the facts bound to the removed cards are cleared.
fn leave(g: &mut Game, id: EffId) -> R {
    let Effect::LeavePlay { p, target, dest, how, source_card, cards, .. } = *g.e(id) else { return Ok(()) };
    match how {
        // An attached card: it leaves the Pokémon's spot for its owner's zone; the Ability locks that read attached cards
        // are re-stamped (`passive::lock_sync_attached`, as after an Attach).
        LeaveHow::Attached => {
            let Some(t) = target else { return Ok(()) };
            crate::engine::cards_zone::relocate(g, t.list(), cards.as_slice(), dest, false);
            crate::spec::passive::lock_sync_attached(g);
            return Ok(());
        }
        // The Stadium: to its owner's zone (a Prism Star card to the Lost Zone); a Stadium can hold an Ability lock.
        LeaveHow::Stadium => {
            let q = g.st.owner(cards.as_slice()[0]) as u8;
            crate::engine::cards_zone::move_physical(g, ListRef::Stadium(q), cards.as_slice(), dest);
            return Ok(());
        }
        _ => {}
    }
    let Some(target) = target else { return Ok(()) };
    let (tp, ts) = (target.p as usize, target.s);
    let removed: SVec<CardId, 60> = {
        let mut v = SVec::new();
        for c in g.st.slot(tp, ts).cards.iter() {
            if g.st.cdef(c).is_pokemon() {
                v.push(c);
            }
        }
        v
    };
    match how {
        LeaveHow::KnockOut => {
            let card = g.st.slot_pokemon(tp, ts);
            let lost_city = g.st.slot(tp, ts).marker.has(LOST_CITY_MARKER) || card.map_or(false, |c| g.st.cdef(c).has_tag(tag::PRISM_STAR));
            if lost_city {
                crate::bail!("LOST_CITY_KO_NOT_PORTED");
            }
            let tools: Vec<CardId> = g.st.slot(tp, ts).tools.iter().collect();
            for t in tools {
                g.move_card_to(target.list(), t, dest);
            }
            crate::engine::game_effect::clear_effects(&mut g.st.players[tp].slots[ts as usize]);
            g.st.players[tp].slots[ts as usize].special_conditions.clear();
            crate::engine::cards_zone::relocate_spot(g, target, dest);
            crate::engine::cards_zone::settle(g);
        }
        LeaveHow::Effect => {
            let _ = source_card;
            crate::engine::cards_zone::relocate_spot(g, target, dest);
            crate::engine::cards_zone::settle(g);
        }
        LeaveHow::BenchShrink => {
            let pokemons = g.st.slot_pokemons(tp, ts);
            let slot = *g.st.slot(tp, ts);
            let others: Vec<CardId> = slot.cards.iter().filter(|c| !g.st.cdef(*c).is_pokemon() && !pokemons.contains(c) && !slot.tools.contains(*c)).collect();
            if !others.is_empty() {
                crate::engine::cards_zone::move_physical(g, target.list(), &others, dest);
            }
            let tools: Vec<CardId> = g.st.slot(tp, ts).tools.iter().collect();
            for t in tools {
                g.move_card_to(target.list(), t, dest);
            }
            if !pokemons.is_empty() {
                crate::engine::cards_zone::move_physical(g, target.list(), pokemons.as_slice(), dest);
            }
        }
        LeaveHow::Attached | LeaveHow::Stadium => unreachable!(),
    }
    let _ = p;
    // A Pokémon that left play is a new object (event-audit #19): its card-bound records go.
    for c in removed.iter() {
        let inst = &mut g.st.cards[*c as usize];
        inst.moved_to_active_this_turn = false;
        inst.damage_taken_last_turn = 0;
    }
    if g.st.slot(tp, ts).cards.is_empty() {
        g.st.players[tp].slots[ts as usize].ko_pending = None;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! The routines against hand-built boards.
    use super::*;
    use crate::cause::CauseKind;
    use serde_json::json;

    const SNORLAX: &str = "Hop's Snorlax JTG 117";
    const MIST: &str = "Mist Energy TEF 161";

    fn game(sc: serde_json::Value) -> Game {
        let mut names: Vec<&str> = Vec::new();
        for n in [SNORLAX, MIST] {
            names.extend(std::iter::repeat(n).take(4));
        }
        while names.len() < 60 {
            names.push("Psychic Energy MEE 5");
        }
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    /// A Knock Out by an effect is recorded and happens at the next state check with the others (user decision D1;
    /// id2089, id810); "prevent all effects of attacks" prevents it (id2427); the Prize card is the opponent's.
    #[test]
    fn effect_knock_out_waits_for_the_state_check() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX, "bench": [{"card": SNORLAX}]}, "opp": {"reset": true, "active": SNORLAX, "bench": [{"card": SNORLAX, "energy": [MIST]}, {"card": SNORLAX}]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let t = SlotRef::new(o, g.st.players[o].active);
        let misty = SlotRef::new(o, g.st.players[o].bench.as_slice()[0]);
        let attack = Cause::new(CauseKind::Attack, None, me as u8);
        assert!(by_effect(&mut g, t, attack).unwrap());
        assert!(g.st.slot_pokemon(o, t.s).is_some(), "still in play until the state check");
        assert!(!by_effect(&mut g, misty, attack).unwrap(), "Mist Energy prevents it");
        g.st.phase = GamePhase::PlayerTurn;
        g.items.clear();
        let left = g.st.players[me].prize_left();
        state_check(&mut g, OnComplete::None).unwrap();
        assert!(g.st.slot_pokemon(o, t.s).is_none(), "Knocked Out at the state check");
        assert_eq!(g.st.players[me].prize_left(), left, "the Prize card is chosen next");
        assert!(g.prompts.iter().any(|p| p.result.is_none() && matches!(p.kind, PromptKind::ChoosePrize { count: 1, .. })));
        assert!(g.st.slot_pokemon(o, misty.s).is_some());
    }

    /// Milotic TWM 50's Mentally Calm refuses the LeavePlay of the opponent's Pokémon to the hand before it happens,
    /// whatever the cause: the Pokémon stays with its card-bound records (id2129); to the deck it leaves.
    #[test]
    fn mentally_calm_refuses_leave_play_to_the_hand() {
        let names = [SNORLAX, "Milotic TWM 50"];
        let mut deck_names: Vec<&str> = Vec::new();
        for n in names {
            deck_names.extend(std::iter::repeat(n).take(4));
        }
        while deck_names.len() < 60 {
            deck_names.push("Psychic Energy MEE 5");
        }
        let deck: Vec<u16> = deck_names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &json!({"me": {"reset": true, "active": SNORLAX, "bench": [{"card": SNORLAX}]}, "opp": {"reset": true, "active": "Milotic TWM 50"}})).unwrap();
        let me = g.st.active_player as usize;
        let t = SlotRef::new(me, g.st.players[me].active);
        let card = g.st.slot_pokemon(me, t.s).unwrap();
        g.st.cards[card as usize].damage_taken_last_turn = 30;
        let trainer = Cause::new(CauseKind::Trainer, None, me as u8);
        assert!(!leave_play(&mut g, t, ListRef::Hand(me as u8), trainer, NO_CARD).unwrap(), "Mentally Calm refuses it");
        assert_eq!(g.st.slot_pokemon(me, t.s), Some(card));
        assert_eq!(g.st.cards[card as usize].damage_taken_last_turn, 30, "nothing happened: no record reset");
        let b = SlotRef::new(me, g.st.players[me].bench.as_slice()[0]);
        assert!(leave_play(&mut g, b, ListRef::Deck(me as u8), trainer, NO_CARD).unwrap(), "into the deck: not prevented");
    }
}
